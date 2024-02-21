"use client"

import { useEffect, useRef, useState } from "react";
import { ServerWSResponse } from "./WebSocketExample";
import { tw } from "~/utils/tw";

export const Canvas=({img_metadata, className}:{img_metadata: ServerWSResponse, className?: string })=>{
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [cursorPosition, setCursorPosition] = useState<[number, number]>()
  const [w, h] = img_metadata.dimension;

  useEffect(() => {
    if (img_metadata){
      const ctx = canvasRef.current?.getContext('2d');
      // ctx?.canvas.width = window.innerWidth;
      // ctx?.canvas.height = window.innerWidth;
      const img = new Image();
      img.src = img_metadata.image;
      [img.width, img.height] = img_metadata.dimension
      img.onload = () => {
          ctx?.drawImage(img, 0, 0)
      }
    }
  }, [img_metadata]);

  return (
    <>
      <canvas
          className={tw("relative w-full h-full rounded-sm", className)}
          ref={canvasRef}
          width={w ?? 1000}
          height={h ?? 1000}
          onPointerMove={(event) => {
            setCursorPosition([Math.round(event.clientX), Math.round(event.clientY)])
          }}
          onPointerLeave={(_)=>{
            setCursorPosition(undefined)
          }}
      />
      <p className="text-sm w-max text-center bg-amber-700 mx-auto rounded-lg px-2 py-1 text-background font-medium">
        cursor position : [{" "} {cursorPosition?.[0] ?? "_"}, {cursorPosition?.[1] ?? "_"}{" "}]
      </p>
    </>
  )
}
