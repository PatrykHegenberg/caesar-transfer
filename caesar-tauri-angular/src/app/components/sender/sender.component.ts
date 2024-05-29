import { Component, ChangeDetectorRef, OnInit  } from '@angular/core';
import { TauriService } from '../../services/tauri.service';
import { Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { FileResponse, open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { StorageService } from '../../services/storage.service';
import { QRCodeModule } from 'angularx-qrcode'

@Component({
  selector: 'app-sender',
  standalone: true,
  imports: [FormsModule, CommonModule, QRCodeModule],
  templateUrl: './sender.component.html',
  styleUrls: ['./sender.component.css']
})
export class SenderComponent implements OnInit {
  files: string[] = [];
  fileNames: string[] = [];
  relayAddress: string = ''; 
  relayPort?: number | null;
  sendingInProgress = false;
  sendingSuccess = false;
  isRelayServerSet = false;
  isUsingShuttle = false;
  transferName: string = "";
  constructor(private tauriService: TauriService, private router: Router, private cdr: ChangeDetectorRef, private storage: StorageService) {
    this.listenToTransferEvents();
  }

  ngOnInit(): void {
    if(this.storage.getLocalEntry('relayServer')) {
      this.isRelayServerSet = true;
      this.relayAddress = this.storage.getLocalEntry('relayServer')
    if(this.storage.getLocalEntry('port')) {
    this.relayPort = this.storage.getLocalEntry('port')
    }
    if(this.storage.getLocalEntry('relayServer') === 'wss://caesar-transfer-iu.shuttleapp.rs') {
        this.isUsingShuttle = true;
      }
    }
}

  redirectToHome() {
    this.router.navigate([''])
  }

  private listenToTransferEvents() {
    listen('transfer_name_event', (event) => {
      this.transferName = event.payload as string; 
      this.cdr.detectChanges();
    })
  }

  reset() {
    this.files = [];
    this.fileNames = [];
    this.sendingInProgress = false;
    this.sendingSuccess = false;
    this.transferName = '';
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
    if(!this.isUsingShuttle) {
    return `ws://${this.relayAddress}:${this.relayPort}`;
    } else {
      return `${this.relayAddress}`
    } 
  }
  sendData() {
    const relay = this.getRelayURL();
    if (this.files.length > 0) {
      this.sendingInProgress = true;
      this.sendingSuccess = false;
      this.tauriService.send(relay, this.files)
        .then(sendDataReturn => {
          this.sendingSuccess = true;
          setTimeout(() => {
            this.reset();
          }, 5000);
        })
        .catch(error => {
          console.error('Error sending data:', error);
          this.sendingSuccess = false;
          this.sendingInProgress = false;
        })
        .finally(() => {
          this.sendingInProgress = false;
        });
    } else {
      console.error('No files to send.');
    }
  }
}