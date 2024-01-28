"use client"

import { useEffect, useState } from 'react';
import Image from 'next/image'
import { Canvas } from './Canvas';

type DisplayType = "image" | "canvas"

export interface ServerWSResponse {
  image: string
  dimension: [number, number]
}

export default function WebSocketExample({port = 8080, display="canvas"}:{port?:number, display:DisplayType}){
  const [imgMetadata, setImageMetadata] = useState<ServerWSResponse>();
  const [socket, setSocket] = useState<WebSocket>()

  useEffect(() => {
    const socket = new WebSocket(`ws://localhost:${port}`)

    socket.onopen = () => {
      setSocket(socket)
      console.log('WebSocket Open');
      socket.send("hello");
    };

    socket.onclose = () => {
      console.log('WebSocket Close');
    };

    socket.onerror = (error) => {
      console.error("WebSocket Error:", error);
    };

    socket.onmessage = (event) => {
      try {
        const data : ServerWSResponse = JSON.parse(event.data);
        console.log("WebSocket Message:", data);
        if (data.image) {
          setImageMetadata(data)
        }
      } catch (e) {
        console.error("Error parsing the WebSocket response:", e);
      }
    };

    return () => {
      socket?.close();
    };
  }, []);

  return (
    <div>
      { (imgMetadata && socket) ? (
        display === "canvas" ? <Canvas img_metadata={imgMetadata} socket={socket}/> : <Image src={imgMetadata.image} alt="Streamed image" objectFit="cover" fill/>
      ) : <p className='text-xl font-semibold'>NEW MEDIA | trying to connect to server...</p>
      }
    </div>
  )
};
