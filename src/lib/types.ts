// ============================================================================
// Keepix — TypeScript Type Definitions
// ============================================================================

/** Represents a culling project/session */
export interface Project {
  id: string;
  name: string;
  root_path: string;
  created_at: string;
  last_opened: string;
  total_items: number;
  thumbnail_path: string | null;
}

/** Represents a single photo or video file */
export interface MediaItem {
  id: string;
  project_id: string;
  file_path: string;
  file_name: string;
  file_type: 'photo' | 'video';
  file_size: number;
  width: number | null;
  height: number | null;
  category_id: number | null;
  star_rating: number;
  color_label: string | null;
  thumbnail_path: string | null;
  preview_path: string | null;
  exif_json: string | null;
  file_hash: string | null;
  date_taken: string | null;
  adjustments: string | null;
  applied_preset: string | null;
  created_at: string;
  updated_at: string;
}

/** Category definition */
export interface Category {
  id: number;
  name: string;
  color: string;
  icon: string;
  hotkey: string;
  sort_order: number;
}

/** EXIF metadata parsed from image */
export interface ExifData {
  camera_make: string | null;
  camera_model: string | null;
  lens_model: string | null;
  software: string | null;
  focal_length: string | null;
  aperture: string | null;
  shutter_speed: string | null;
  iso: string | null;
  exposure_compensation: string | null;
  flash: string | null;
  metering_mode: string | null;
  white_balance: string | null;
  width: number | null;
  height: number | null;
  orientation: number | null;
  color_space: string | null;
  date_taken: string | null;
  date_digitized: string | null;
  gps_latitude: number | null;
  gps_longitude: number | null;
  gps_altitude: number | null;
  file_size: number | null;
  file_format: string | null;
  duration: number | null;
  unsupported: boolean | null;
}

/** Scan progress event from backend */
export interface ScanProgress {
  phase: 'scanning' | 'importing' | 'thumbnails' | 'complete';
  current: number;
  total: number;
  current_file: string;
}

/** Scan result from backend */
export interface ScanResult {
  project_id: string;
  total_files: number;
  photos: number;
  videos: number;
}

/** Category statistics */
export interface CategoryStats {
  category_id: number | null;
  category_name: string;
  count: number;
}

/** Paginated media response */
export interface PaginatedMedia {
  items: MediaItem[];
  total: number;
  page: number;
  page_size: number;
}

/** View modes for the culling workspace */
export type ViewMode = 'grid' | 'preview';

/** Sort options */
export type SortBy = 'name' | 'date' | 'size' | 'rating' | 'type';
export type SortOrder = 'asc' | 'desc';

/** Visual diagnostics modes */
export type DiagnosticsMode = 'none' | 'peaking' | 'zebra';

/** Represents a group of similar/burst photos */
export interface BurstGroup {
  id: string;
  leadItem: MediaItem;
  items: MediaItem[];
}


/** Color labels available */
export const COLOR_LABELS = [
  { value: 'red', label: 'Red', hex: '#ef4444' },
  { value: 'orange', label: 'Orange', hex: '#f97316' },
  { value: 'yellow', label: 'Yellow', hex: '#eab308' },
  { value: 'green', label: 'Green', hex: '#22c55e' },
  { value: 'blue', label: 'Blue', hex: '#3b82f6' },
  { value: 'purple', label: 'Purple', hex: '#a855f7' },
] as const;

/** Default categories with their visual properties */
export const DEFAULT_CATEGORIES: Category[] = [
  { id: 1, name: 'Buang', color: '#ef4444', icon: '🗑️', hotkey: '1', sort_order: 1 },
  { id: 2, name: 'Simpan', color: '#22c55e', icon: '⭐', hotkey: '2', sort_order: 2 },
  { id: 3, name: 'Simpan Draft', color: '#f59e0b', icon: '📋', hotkey: '3', sort_order: 3 },
  { id: 4, name: 'Review', color: '#6366f1', icon: '👁️', hotkey: '4', sort_order: 4 },
];

/** Format file size to human readable string */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

/** Get category badge class from category id */
export function getCategoryClass(categoryId: number | null): string {
  switch (categoryId) {
    case 1: return 'badge-buang';
    case 2: return 'badge-simpan';
    case 3: return 'badge-draft';
    case 4: return 'badge-review';
    default: return '';
  }
}

/** Get category name from id */
export function getCategoryName(categoryId: number | null): string {
  const cat = DEFAULT_CATEGORIES.find(c => c.id === categoryId);
  return cat?.name ?? 'Uncategorized';
}

/** Get category color from id */
export function getCategoryColor(categoryId: number | null): string {
  const cat = DEFAULT_CATEGORIES.find(c => c.id === categoryId);
  return cat?.color ?? 'transparent';
}

/** Unique EXIF filters returned by backend */
export interface ExifFilters {
  camera_models: string[];
  lens_models: string[];
  color_labels: string[];
}

/** Export progress event from backend */
export interface ExportProgress {
  phase: 'exporting' | 'complete' | 'error';
  current: number;
  total: number;
  current_file: string;
  status: string;
}
