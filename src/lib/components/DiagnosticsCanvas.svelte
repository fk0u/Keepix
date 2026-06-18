<script lang="ts">
  import type { DiagnosticsMode } from '$lib/types';

  let {
    src,
    mode = 'none',
    style = '',
    focusColor = 'green',
    focusSensitivity = 'medium',
    gpuAccel = true,
  }: {
    src: string;
    mode: DiagnosticsMode;
    style?: string;
    focusColor?: string;
    focusSensitivity?: string;
    gpuAccel?: boolean;
  } = $props();

  let canvasEl = $state<HTMLCanvasElement | null>(null);
  let loading = $state(false);
  let imageObj: HTMLImageElement | null = null;

  $effect(() => {
    if (src) {
      loadImage(src);
    }
  });

  $effect(() => {
    // Re-apply filter if diagnostics parameters or mode changes
    const _mode = mode;
    const _color = focusColor;
    const _sens = focusSensitivity;
    const _gpu = gpuAccel;
    if (imageObj && canvasEl) {
      applyDiagnostics();
    }
  });

  function loadImage(imageUrl: string) {
    loading = true;
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.src = imageUrl;
    img.onload = () => {
      imageObj = img;
      loading = false;
      applyDiagnostics();
    };
    img.onerror = () => {
      loading = false;
      imageObj = null;
    };
  }

  function applyDiagnostics() {
    if (!canvasEl || !imageObj) return;

    // Limit maximum size of canvas for focus peaking / clipping to ensure high performance
    const maxDim = 960;
    let w = imageObj.width;
    let h = imageObj.height;

    if (w > maxDim || h > maxDim) {
      if (w > h) {
        h = Math.round((h * maxDim) / w);
        w = maxDim;
      } else {
        w = Math.round((w * maxDim) / h);
        h = maxDim;
      }
    }

    canvasEl.width = w;
    canvasEl.height = h;

    if (mode === 'none') {
      // Direct drawing to 2D canvas is fastest if no diagnostics are requested
      renderCanvas2D(w, h);
      return;
    }

    if (gpuAccel) {
      try {
        const success = renderWebGL(w, h);
        if (success) return;
      } catch (err) {
        console.warn('WebGL render failed, falling back to 2D canvas:', err);
      }
    }

    // Fallback if WebGL fails or is disabled
    renderCanvas2D(w, h);
  }

  function renderWebGL(width: number, height: number): boolean {
    if (!canvasEl || !imageObj) return false;

    // Get WebGL context
    const gl = canvasEl.getContext('webgl2') || 
               canvasEl.getContext('webgl') || 
               (canvasEl.getContext('experimental-webgl') as WebGLRenderingContext | null);
    if (!gl) return false;

    // Clear viewport
    gl.viewport(0, 0, width, height);
    gl.clearColor(0.0, 0.0, 0.0, 0.0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // Shaders Sources
    const vsSource = `
      attribute vec2 a_position;
      varying vec2 v_texCoord;
      void main() {
        gl_Position = vec4(a_position, 0.0, 1.0);
        // Map positions [-1, 1] to texture coords [0, 1]
        v_texCoord = a_position * 0.5 + 0.5;
        // Flip Y coordinates to match standard 2D canvas orientation
        v_texCoord.y = 1.0 - v_texCoord.y;
      }
    `;

    let fsSource = '';
    if (mode === 'peaking') {
      fsSource = `
        precision mediump float;
        varying vec2 v_texCoord;
        uniform sampler2D u_image;
        uniform vec2 u_textureSize;
        uniform float u_threshold;
        uniform vec3 u_peakColor;

        float luma(vec3 color) {
          return 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
        }

        void main() {
          vec2 onePixel = vec2(1.0) / u_textureSize;
          
          float c00 = luma(texture2D(u_image, v_texCoord + onePixel * vec2(-1.0, -1.0)).rgb);
          float c10 = luma(texture2D(u_image, v_texCoord + onePixel * vec2( 0.0, -1.0)).rgb);
          float c20 = luma(texture2D(u_image, v_texCoord + onePixel * vec2( 1.0, -1.0)).rgb);
          
          float c01 = luma(texture2D(u_image, v_texCoord + onePixel * vec2(-1.0,  0.0)).rgb);
          float c21 = luma(texture2D(u_image, v_texCoord + onePixel * vec2( 1.0,  0.0)).rgb);
          
          float c02 = luma(texture2D(u_image, v_texCoord + onePixel * vec2(-1.0,  1.0)).rgb);
          float c12 = luma(texture2D(u_image, v_texCoord + onePixel * vec2( 0.0,  1.0)).rgb);
          float c22 = luma(texture2D(u_image, v_texCoord + onePixel * vec2( 1.0,  1.0)).rgb);
          
          float gx = -1.0 * c00 + 1.0 * c20 +
                     -2.0 * c01 + 2.0 * c21 +
                     -1.0 * c02 + 1.0 * c22;
                     
          float gy = -1.0 * c00 - 2.0 * c10 - 1.0 * c20 +
                      1.0 * c02 + 2.0 * c12 + 1.0 * c22;
                      
          float mag = sqrt(gx * gx + gy * gy);
          
          vec4 originalColor = texture2D(u_image, v_texCoord);
          
          if (mag > u_threshold) {
            gl_FragColor = vec4(u_peakColor, 1.0);
          } else {
            gl_FragColor = vec4(originalColor.rgb * 0.65, originalColor.a);
          }
        }
      `;
    } else if (mode === 'zebra') {
      fsSource = `
        precision mediump float;
        varying vec2 v_texCoord;
        uniform sampler2D u_image;
        void main() {
          vec4 color = texture2D(u_image, v_texCoord);
          if (color.r >= 0.988 && color.g >= 0.988 && color.b >= 0.988) {
            gl_FragColor = vec4(0.937, 0.266, 0.266, 1.0);
          } else if (color.r <= 0.015 && color.g <= 0.015 && color.b <= 0.015) {
            gl_FragColor = vec4(0.231, 0.51, 0.965, 1.0);
          } else {
            gl_FragColor = color;
          }
        }
      `;
    } else {
      fsSource = `
        precision mediump float;
        varying vec2 v_texCoord;
        uniform sampler2D u_image;
        void main() {
          gl_FragColor = texture2D(u_image, v_texCoord);
        }
      `;
    }

    // Compile VS
    const vs = gl.createShader(gl.VERTEX_SHADER);
    if (!vs) return false;
    gl.shaderSource(vs, vsSource);
    gl.compileShader(vs);
    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS)) {
      console.error('VS Error:', gl.getShaderInfoLog(vs));
      return false;
    }

    // Compile FS
    const fs = gl.createShader(gl.FRAGMENT_SHADER);
    if (!fs) return false;
    gl.shaderSource(fs, fsSource);
    gl.compileShader(fs);
    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS)) {
      console.error('FS Error:', gl.getShaderInfoLog(fs));
      return false;
    }

    // Link Program
    const program = gl.createProgram();
    if (!program) return false;
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error('Program Link Error:', gl.getProgramInfoLog(program));
      return false;
    }
    gl.useProgram(program);

    // Setup coordinates buffer (full screen quad)
    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    const positions = new Float32Array([
      -1.0, -1.0,
       1.0, -1.0,
      -1.0,  1.0,
      -1.0,  1.0,
       1.0, -1.0,
       1.0,  1.0,
    ]);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

    const positionLocation = gl.getAttribLocation(program, 'a_position');
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);

    // Setup texture
    const texture = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

    // Upload image
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, imageObj);

    // Set texture uniform
    const uImageLoc = gl.getUniformLocation(program, 'u_image');
    gl.uniform1i(uImageLoc, 0);

    if (mode === 'peaking') {
      const uTextureSizeLoc = gl.getUniformLocation(program, 'u_textureSize');
      gl.uniform2f(uTextureSizeLoc, width, height);

      // Determine threshold based on focusSensitivity
      let threshold = 38.0 / 255.0;
      if (focusSensitivity === 'low') threshold = 55.0 / 255.0;
      else if (focusSensitivity === 'high') threshold = 22.0 / 255.0;
      
      const uThresholdLoc = gl.getUniformLocation(program, 'u_threshold');
      gl.uniform1f(uThresholdLoc, threshold);

      // Determine peak color
      let rPeak = 34.0/255.0, gPeak = 197.0/255.0, bPeak = 94.0/255.0;
      if (focusColor === 'red') {
        rPeak = 239.0/255.0; gPeak = 68.0/255.0; bPeak = 68.0/255.0;
      } else if (focusColor === 'green') {
        rPeak = 34.0/255.0; gPeak = 197.0/255.0; bPeak = 94.0/255.0;
      } else if (focusColor === 'blue') {
        rPeak = 59.0/255.0; gPeak = 130.0/255.0; bPeak = 246.0/255.0;
      } else if (focusColor === 'yellow') {
        rPeak = 234.0/255.0; gPeak = 179.0/255.0; bPeak = 8.0/255.0;
      } else if (focusColor === 'cyan') {
        rPeak = 6.0/255.0; gPeak = 182.0/255.0; bPeak = 212.0/255.0;
      }

      const uPeakColorLoc = gl.getUniformLocation(program, 'u_peakColor');
      gl.uniform3f(uPeakColorLoc, rPeak, gPeak, bPeak);
    }

    // Draw
    gl.drawArrays(gl.TRIANGLES, 0, 6);

    // Clean up
    gl.deleteTexture(texture);
    gl.deleteBuffer(positionBuffer);
    gl.deleteProgram(program);
    gl.deleteShader(vs);
    gl.deleteShader(fs);

    return true;
  }

  function renderCanvas2D(width: number, height: number) {
    if (!canvasEl || !imageObj) return;
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    // Draw image
    ctx.drawImage(imageObj, 0, 0, width, height);

    if (mode === 'none') {
      return; // Normal drawing is sufficient
    }

    try {
      const imgData = ctx.getImageData(0, 0, width, height);
      const input = imgData.data;
      const outputImgData = ctx.createImageData(width, height);
      const output = outputImgData.data;

      if (mode === 'peaking') {
        runFocusPeaking(width, height, input, output);
      } else if (mode === 'zebra') {
        runExposureZebra(width, height, input, output);
      }

      ctx.putImageData(outputImgData, 0, 0);
    } catch (e) {
      console.error('Failed to apply diagnostics 2D filter:', e);
    }
  }

  function runFocusPeaking(width: number, height: number, input: Uint8ClampedArray, output: Uint8ClampedArray) {
    // Sobel Kernels
    const gx = [
      [-1, 0, 1],
      [-2, 0, 2],
      [-1, 0, 1]
    ];
    const gy = [
      [-1, -2, -1],
      [0, 0, 0],
      [1, 2, 1]
    ];

    let threshold = 38;
    if (focusSensitivity === 'low') threshold = 55;
    else if (focusSensitivity === 'high') threshold = 22;

    let rPeak = 34, gPeak = 197, bPeak = 94;
    if (focusColor === 'red') {
      rPeak = 239; gPeak = 68; bPeak = 68;
    } else if (focusColor === 'green') {
      rPeak = 34; gPeak = 197; bPeak = 94;
    } else if (focusColor === 'blue') {
      rPeak = 59; gPeak = 130; bPeak = 246;
    } else if (focusColor === 'yellow') {
      rPeak = 234; gPeak = 179; bPeak = 8;
    } else if (focusColor === 'cyan') {
      rPeak = 6; gPeak = 182; bPeak = 212;
    }

    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const idx = (y * width + x) * 4;

        if (y === 0 || y === height - 1 || x === 0 || x === width - 1) {
          output[idx] = input[idx] * 0.5;
          output[idx + 1] = input[idx + 1] * 0.5;
          output[idx + 2] = input[idx + 2] * 0.5;
          output[idx + 3] = 255;
          continue;
        }

        let valX = 0;
        let valY = 0;

        for (let ky = -1; ky <= 1; ky++) {
          for (let kx = -1; kx <= 1; kx++) {
            const pixelIdx = ((y + ky) * width + (x + kx)) * 4;
            const r = input[pixelIdx];
            const g = input[pixelIdx + 1];
            const b = input[pixelIdx + 2];
            const luma = 0.299 * r + 0.587 * g + 0.114 * b;

            valX += luma * gx[ky + 1][kx + 1];
            valY += luma * gy[ky + 1][kx + 1];
          }
        }

        const mag = Math.sqrt(valX * valX + valY * valY);

        if (mag > threshold) {
          output[idx] = rPeak;
          output[idx + 1] = gPeak;
          output[idx + 2] = bPeak;
          output[idx + 3] = 255;
        } else {
          output[idx] = Math.round(input[idx] * 0.65);
          output[idx + 1] = Math.round(input[idx + 1] * 0.65);
          output[idx + 2] = Math.round(input[idx + 2] * 0.65);
          output[idx + 3] = 255;
        }
      }
    }
  }

  function runExposureZebra(width: number, height: number, input: Uint8ClampedArray, output: Uint8ClampedArray) {
    for (let i = 0; i < input.length; i += 4) {
      const r = input[i];
      const g = input[i + 1];
      const b = input[i + 2];

      if (r >= 252 && g >= 252 && b >= 252) {
        output[i] = 239;
        output[i + 1] = 68;
        output[i + 2] = 68;
        output[i + 3] = 255;
      }
      else if (r <= 4 && g <= 4 && b <= 4) {
        output[i] = 59;
        output[i + 1] = 130;
        output[i + 2] = 246;
        output[i + 3] = 255;
      }
      else {
        output[i] = r;
        output[i + 1] = g;
        output[i + 2] = b;
        output[i + 3] = 255;
      }
    }
  }
</script>

<div class="canvas-container">
  <canvas
    bind:this={canvasEl}
    class="preview-diagnostics-canvas"
    {style}
  ></canvas>
  {#if loading}
    <div class="canvas-loader">
      <div class="loader-spinner"></div>
    </div>
  {/if}
</div>

<style>
  .canvas-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .preview-diagnostics-canvas {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    transition: transform 0.1s ease;
    user-select: none;
  }

  .canvas-loader {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.4);
    z-index: 10;
  }

  .loader-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-radius: 50%;
    border-top-color: var(--accent);
    animation: spin 1s ease-in-out infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
