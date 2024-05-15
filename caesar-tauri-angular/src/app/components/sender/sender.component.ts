import { Component } from '@angular/core';
import { TauriService } from '../../services/tauri.service';
import { Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { FileResponse, open } from '@tauri-apps/plugin-dialog';

@Component({
  selector: 'app-sender',
  standalone: true,
  imports: [FormsModule, CommonModule],
  templateUrl: './sender.component.html',
  styleUrls: ['./sender.component.css']
})
export class SenderComponent {
  files: string[] = [];
  fileNames: string[] = [];
  relayAddress: string = ''; 
  relayPort?: number;  
  constructor(private tauriService: TauriService, private router: Router) {}

  redirectToHome() {
    this.router.navigate([''])
  }

  async selectFile() {
    // Open the file dialog and get the file path(s)
    const selected:any = await open({
      multiple: false
    });
    this.fileNames.push(selected.name);
    this.files.push(selected.path);
  }
  

  getRelayURL(): string {
    return `ws://${this.relayAddress}:${this.relayPort}`;
  }

  sendData() {
    const relay = this.getRelayURL();
    if (this.files.length > 0) {
      this.tauriService.send(relay, this.files)
        .then(sendDataReturn => console.log(sendDataReturn + ' Data sent successfully'))
        .catch(error => console.error('Error sending data:', error));
    } else {
      console.error('No files to send.');
    }
  }
}