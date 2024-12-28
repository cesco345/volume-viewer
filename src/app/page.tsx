"use client";

import dynamic from "next/dynamic";

const VolumeViewer = dynamic(() => import("@/components/viewer/VolumeViewer"), {
  ssr: false,
});

export default function Home() {
  return (
    <main className="min-h-screen p-4">
      <div className="container mx-auto">
        <h1 className="text-2xl font-bold mb-4">Volume Viewer</h1>
        <div className="h-[800px] border rounded-lg overflow-hidden bg-white">
          <VolumeViewer />
        </div>
      </div>
    </main>
  );
}
