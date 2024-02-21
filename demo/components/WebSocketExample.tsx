"use client"

import { useEffect, useState } from 'react';
import { Canvas } from './Canvas';
import { Controller } from './Controller';
import { LoadingScreen } from './LoadingPlaceholder';

export interface ServerWSResponse {
  image: string
  dimension: [number, number]
  aspect_ratio: [number, number]
}

export default function WebSocketExample({port = 8080}:{port?:number}){
  const server_ws_url  = process.env.NODE_ENV != "production" ? `ws://localhost:${port}` : "wss://new-media.fly.dev"
  const [imgMetadata, setImageMetadata] = useState<ServerWSResponse>();
  const [socket, setSocket] = useState<WebSocket>()

  useEffect(() => {
    const socket = new WebSocket(server_ws_url)

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
    <>
      { imgMetadata ? (
          <div className={`w-screen px-4 aspect-[${imgMetadata.aspect_ratio[0]}/${imgMetadata.aspect_ratio[1]}]`}>
            <Canvas img_metadata={imgMetadata} className='w-full h-full' />
          </div>
      ) : (<LoadingScreen/>)
      }
      {socket ? <Controller socket={socket}/> : null}
    </>
  )
};
