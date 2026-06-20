// ============================================================================
// Keepix — Video Processor & Thumbnail Extractor Service
// ============================================================================

import { invoke } from '@tauri-apps/api/core';
import { toAssetUrl } from '$lib/services/tauri-bridge';
import { mediaItems, loadMediaItems, loadCategoryStats } from '$lib/stores/media';
import type { MediaItem } from '$lib/types';

let queue: MediaItem[] = [];
let isProcessing = false;
let currentProjectId = '';

/**
 * Scan media items and queue video files that need metadata extraction.
 */
export function startVideoProcessingQueue(items: MediaItem[], projectId: string) {
  currentProjectId = projectId;
  
  // Filter for videos that haven't been processed yet (width is null or 0)
  // width === -1 indicates we already tried and the format is unsupported in WebView
  const pendingVideos = items.filter(
    (item) => item.file_type === 'video' && (item.width === null || item.width === 0)
  );

  if (pendingVideos.length === 0) {
    return;
  }

  // Add new items to queue, avoiding duplicates
  for (const video of pendingVideos) {
    if (!queue.some((q) => q.id === video.id)) {
      queue.push(video);
    }
  }

  if (!isProcessing) {
    processNext();
  }
}

/**
 * Process the next video in the queue.
 */
async function processNext() {
  if (queue.length === 0) {
    isProcessing = false;
    return;
  }

  isProcessing = true;
  const item = queue.shift();
  if (!item) {
    processNext();
    return;
  }

  try {
    const videoUrl = toAssetUrl(item.file_path);
    const result = await extractVideoMetadataAndFrame(videoUrl);
    
    // Send to backend
    await invoke('save_video_metadata', {
      mediaId: item.id,
      width: result.width,
      height: result.height,
      duration: result.duration,
      thumbnailBase64: result.thumbnailBase64,
    });

    // Update local store immediately for reactivity
    mediaItems.update((items) =>
      items.map((it) =>
        it.id === item.id
          ? {
              ...it,
              width: result.width,
              height: result.height,
              thumbnail_path: it.thumbnail_path || 'updated', // Force refresh trigger
            }
          : it
      )
    );

    // Refresh project details
    if (currentProjectId) {
      await loadMediaItems(currentProjectId);
      await loadCategoryStats(currentProjectId);
    }
  } catch (err) {
    console.warn(`Failed to extract thumbnail/metadata for video ${item.file_name}:`, err);
    // Mark as unsupported on the backend so we don't try it again
    try {
      await invoke('mark_video_unsupported', { mediaId: item.id });
      
      // Update local store so UI shows unsupported status
      mediaItems.update((items) =>
        items.map((it) =>
          it.id === item.id ? { ...it, width: -1, height: -1 } : it
        )
      );
    } catch (dbErr) {
      console.error('Failed to mark video as unsupported:', dbErr);
    }
  }

  // Yield to main thread, then process next item
  setTimeout(processNext, 100);
}

interface ExtractionResult {
  width: number;
  height: number;
  duration: number;
  thumbnailBase64: string;
}

/**
 * Loads video in a hidden DOM element, seeks to 1.0 second, and captures frame via canvas.
 */
function extractVideoMetadataAndFrame(videoUrl: string): Promise<ExtractionResult> {
  return new Promise((resolve, reject) => {
    const video = document.createElement('video');
    video.src = videoUrl;
    video.preload = 'metadata';
    video.muted = true;
    video.playsInline = true;
    
    // Set a timeout to prevent infinite hangs on corrupted files
    const timeoutId = setTimeout(() => {
      cleanup();
      reject(new Error('Video loading timed out'));
    }, 10000);

    function cleanup() {
      clearTimeout(timeoutId);
      video.removeEventListener('loadedmetadata', onMetadata);
      video.removeEventListener('seeked', onSeeked);
      video.removeEventListener('error', onError);
      video.removeAttribute('src');
      video.load();
    }

    function onError(e: any) {
      cleanup();
      reject(new Error('Failed to load video codec or container'));
    }

    function onMetadata() {
      // Once metadata is loaded, seek to 1.0s or half duration (whichever is less)
      const seekTime = Math.min(1.0, video.duration / 2);
      video.currentTime = seekTime;
    }

    function onSeeked() {
      try {
        const width = video.videoWidth || 640;
        const height = video.videoHeight || 360;
        const duration = video.duration || 0;

        // Create canvas to draw the frame
        const canvas = document.createElement('canvas');
        // Max size for thumbnails to optimize database/file storage size
        const maxDim = 400;
        let w = width;
        let h = height;
        if (width > height) {
          if (width > maxDim) {
            h = Math.round((maxDim * height) / width);
            w = maxDim;
          }
        } else {
          if (height > maxDim) {
            w = Math.round((maxDim * width) / height);
            h = maxDim;
          }
        }

        canvas.width = w;
        canvas.height = h;

        const ctx = canvas.getContext('2d');
        if (!ctx) {
          throw new Error('Could not get 2D canvas context');
        }

        ctx.drawImage(video, 0, 0, w, h);
        const dataUrl = canvas.toDataURL('image/webp', 0.85);

        cleanup();
        resolve({
          width,
          height,
          duration,
          thumbnailBase64: dataUrl,
        });
      } catch (err) {
        cleanup();
        reject(err);
      }
    }

    video.addEventListener('loadedmetadata', onMetadata);
    video.addEventListener('seeked', onSeeked);
    video.addEventListener('error', onError);
    
    // Start loading
    video.load();
  });
}
