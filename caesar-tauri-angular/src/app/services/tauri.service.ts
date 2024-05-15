import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

@Injectable({
  providedIn: 'root'
})
export class TauriService {

  constructor() { }

  async send(relay: string, files: string[]): Promise<any> {
    console.log(relay, files);
    await invoke('send', { relay, files });
  }

  serve(port: number, listen_addr: string): Promise<any> {
    console.log("Listening on address:" + listen_addr + ":" + port);
    return invoke('serve', { port: port, listen_addr: listen_addr });
  }
}