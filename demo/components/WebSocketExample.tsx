"use client"

import { useEffect, useState } from 'react';
import Image from 'next/image'

export default function WebSocketExample({port = 8080}:{port?:number}){
  const [imgUrl, setImgUrl] = useState<string|null>(null);
  const [socket, setSocket] = useState<WebSocket|null>(null)
  
  useEffect(() => {
    const socket = new WebSocket(`ws://localhost:${port}`)
    
    setSocket(socket);

    socket.onopen = () => {
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
        const data = JSON.parse(event.data);
        console.log("WebSocket Message:", data);
        if (data.image) {
          setImgUrl(data.image);
        }
      } catch (e) {
        console.error("Error parsing the WebSocket response:", e);
      }
    };

    return () => {
      socket.close();
    };
  }, []);
  
  return (
    <div>
      {imgUrl ? (
        <div className="h-[80vh] w-[80vw] relative">
            <Image
              className="rounded"
              src={imgUrl}
              alt="Streamed image"
              objectFit="cover"
              fill
              />
          </div>
      ): (
        <p>loading ...</p>
        )}
    </div>
  )
};

// {/* <img style={{border: "1px solid black"}} src={imgUrl} alt="Received from server" /> */}