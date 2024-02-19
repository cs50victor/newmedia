# new_media

## demo
![screenshot of demo](.github/assets/screenshot.png)

- run web: `pnpm --dir=demo dev`
- run server: `docker run -it -p 8080:8080 $(docker build -f Dockerfile.dev -q .)`

```
- only "audio" feature MUST be real-time "lipsync"
- focus on primarily building realistic camera features not visual effects existing video editing programs (i.e Davinci) work on.
```