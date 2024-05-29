import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

@Injectable({
  providedIn: 'root'
})
export class TauriService {

  constructor() { }

  async send(relay: string, files: string[]): Promise<any> {
    await invoke('send', { relay, files });
  }

  async serve(port: number, listen_addr: string): Promise<any> {
    return invoke('serve', { port: port, listen_addr: listen_addr });
  }

  async receive(relay: string, name: string) {
    return invoke('receive',{relay: relay, name: name});
  }
}