import React, { useEffect, useRef, useState, useCallback } from "react";
import { Upload } from "lucide-react";

interface ViewerState {
  isLoading: boolean;
  error: string | null;
  isDragging: boolean;
  lastMousePos: { x: number; y: number };
  mouseButton: number | null;
  dimensions: [number, number, number] | null;
}

interface VolumeViewerWasm {
  load_volume(data: Uint8Array): Promise<number[]>;
  render(): Uint8Array;
  orbit(deltaTheta: number, deltaPhi: number): void;
  zoom(delta: number): void;
  pan(delta: [number, number]): void;
  resize(width: number, height: number): void;
}

const VolumeViewer: React.FC = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationFrameRef = useRef<number>();
  const [viewer, setViewer] = useState<VolumeViewerWasm | null>(null);
  const [state, setState] = useState<ViewerState>({
    isLoading: true,
    error: null,
    isDragging: false,
    lastMousePos: { x: 0, y: 0 },
    mouseButton: null,
    dimensions: null,
  });

  const renderFrame = useCallback(() => {
    if (!viewer || !canvasRef.current) return;

    try {
      const canvas = canvasRef.current;
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        throw new Error("Failed to get 2D context");
      }

      const frameBuffer = viewer.render();
      const imageData = new ImageData(
        new Uint8ClampedArray(frameBuffer),
        canvas.width,
        canvas.height
      );
      ctx.putImageData(imageData, 0, 0);
    } catch (error) {
      console.error("Render error:", error);
      setState((prev) => ({
        ...prev,
        error: error instanceof Error ? error.message : "Render failed",
      }));
    }
  }, [viewer]);

  useEffect(() => {
    const initViewer = async () => {
      try {
        const wasmModule = await import(
          /* webpackIgnore: true */ "/pkg/volume_viewer.js"
        );
        await wasmModule.default("/pkg/volume_viewer_bg.wasm");

        const canvas = canvasRef.current;
        if (!canvas) throw new Error("Canvas not found");

        const viewerInstance = new wasmModule.VolumeViewer(
          canvas.width,
          canvas.height
        );
        setViewer(viewerInstance);
        setState((prev) => ({ ...prev, isLoading: false }));
      } catch (error) {
        console.error("Initialization error:", error);
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error:
            error instanceof Error
              ? error.message
              : "Failed to initialize viewer",
        }));
      }
    };

    initViewer();

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  useEffect(() => {
    const handleResize = () => {
      if (!viewer || !canvasRef.current) return;

      const canvas = canvasRef.current;
      const container = canvas.parentElement;
      if (!container) return;

      try {
        const { width, height } = container.getBoundingClientRect();
        canvas.width = width;
        canvas.height = height;
        viewer.resize(width, height);
        renderFrame();
      } catch (error) {
        console.error("Resize error:", error);
        setState((prev) => ({
          ...prev,
          error: error instanceof Error ? error.message : "Resize failed",
        }));
      }
    };

    window.addEventListener("resize", handleResize);
    handleResize(); // Initial resize

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, [viewer, renderFrame]);

  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    setState((prev) => ({
      ...prev,
      isDragging: true,
      lastMousePos: { x: e.clientX, y: e.clientY },
      mouseButton: e.button,
      error: null, // Clear any previous errors
    }));
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!state.isDragging || !viewer) return;

    try {
      const deltaX = e.clientX - state.lastMousePos.x;
      const deltaY = e.clientY - state.lastMousePos.y;

      if (state.mouseButton === 0) {
        // Left button - Orbit
        viewer.orbit(deltaX * 0.005, deltaY * 0.005);
      } else if (state.mouseButton === 2) {
        // Right button - Pan
        viewer.pan([deltaX * 0.1, deltaY * 0.1]);
      }

      setState((prev) => ({
        ...prev,
        lastMousePos: { x: e.clientX, y: e.clientY },
        error: null,
      }));

      renderFrame();
    } catch (error) {
      console.error("Mouse move error:", error);
      setState((prev) => ({
        ...prev,
        error:
          error instanceof Error ? error.message : "Mouse interaction failed",
        isDragging: false,
      }));
    }
  };

  const handleMouseUp = () => {
    setState((prev) => ({
      ...prev,
      isDragging: false,
      mouseButton: null,
    }));
  };

  const handleWheel = (e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    if (!viewer) return;

    try {
      const delta = e.deltaY > 0 ? 0.1 : -0.1;
      viewer.zoom(delta);
      renderFrame();
    } catch (error) {
      console.error("Wheel error:", error);
      setState((prev) => ({
        ...prev,
        error: error instanceof Error ? error.message : "Zoom failed",
      }));
    }
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
  };

  const handleFileUpload = async (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    const file = event.target.files?.[0];
    if (!file || !viewer) return;

    try {
      setState((prev) => ({
        ...prev,
        isLoading: true,
        error: null,
        dimensions: null, // Clear previous dimensions
      }));

      // Validate file type
      if (
        !file.name.toLowerCase().endsWith(".tif") &&
        !file.name.toLowerCase().endsWith(".tiff")
      ) {
        throw new Error("Please select a TIFF file");
      }

      // Validate file size
      const MAX_FILE_SIZE = 256 * 1024 * 1024; // 256MB
      if (file.size > MAX_FILE_SIZE) {
        throw new Error("File size too large (max 256MB)");
      }

      // Start fresh render loop
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }

      const arrayBuffer = await file.arrayBuffer();
      const uint8Array = new Uint8Array(arrayBuffer);

      const dimensions = await viewer.load_volume(uint8Array);

      // Validate dimensions
      if (!dimensions || dimensions.length !== 3) {
        throw new Error("Invalid volume dimensions received");
      }

      setState((prev) => ({
        ...prev,
        dimensions: dimensions as [number, number, number],
        isLoading: false,
        error: null,
      }));

      // Ensure canvas is properly sized
      const canvas = canvasRef.current;
      if (canvas) {
        const container = canvas.parentElement;
        if (container) {
          const { width, height } = container.getBoundingClientRect();
          canvas.width = width;
          canvas.height = height;
          viewer.resize(width, height);
        }
      }

      renderFrame();
    } catch (error) {
      console.error("Error loading file:", error);
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : "Failed to load image",
        dimensions: null, // Clear dimensions on error
      }));
    } finally {
      // Clear file input to allow reloading same file
      event.target.value = "";
    }
  };

  return (
    <div className="w-full h-full flex flex-col">
      <div className="bg-white border-b px-4 py-2 flex items-center justify-between">
        <label className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded cursor-pointer hover:bg-blue-600 transition-colors">
          <Upload size={20} />
          Load TIFF Image
          <input
            type="file"
            accept=".tiff,.tif"
            onChange={handleFileUpload}
            className="hidden"
            disabled={state.isLoading}
          />
        </label>
        {state.dimensions && (
          <div className="text-sm text-gray-600">
            Dimensions: {state.dimensions[0]} x {state.dimensions[1]} x{" "}
            {state.dimensions[2]}
          </div>
        )}
      </div>

      <div className="flex-1 relative bg-gray-50">
        {state.isLoading && (
          <div className="absolute inset-0 flex items-center justify-center bg-white bg-opacity-75 z-10">
            <div className="flex flex-col items-center gap-2">
              <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
              <div className="text-gray-600">Loading...</div>
            </div>
          </div>
        )}

        {state.error && (
          <div className="absolute top-4 right-4 z-20 max-w-md bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
            {state.error}
          </div>
        )}

        <canvas
          ref={canvasRef}
          width={800}
          height={600}
          className="w-full h-full"
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          onWheel={handleWheel}
          onContextMenu={handleContextMenu}
        />
      </div>
    </div>
  );
};

export default VolumeViewer;
