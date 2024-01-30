"use client"

import { useEffect, useRef, useState } from "react";
import { ServerWSResponse } from "./WebSocketExample";

export const Canvas=({img_metadata}:{img_metadata: ServerWSResponse })=>{
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [cursorPosition, setCursorPosition] = useState<[number, number]>()
  const aspectRatio = 16/9;

  useEffect(() => {
    if (img_metadata){
      const ctx = canvasRef.current?.getContext('2d');
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
          className="w-screen h-screen"
          ref={canvasRef}
          width={1000}
          height={1000}
          onPointerMove={(event) => {
            const [x,y]  = [event.clientX, event.clientY]
            setCursorPosition([Math.round(x),Math.round(y)])
            console.log(event.clientX, event.clientY)
          }}
          onPointerLeave={(_)=>{
            setCursorPosition(undefined)
          }}
      />
    </>
  )
}
