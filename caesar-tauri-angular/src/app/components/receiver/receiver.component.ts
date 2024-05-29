import { Component, OnInit } from '@angular/core';
import { TauriService } from '../../services/tauri.service';
import { Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { StorageService } from '../../services/storage.service';

@Component({
  selector: 'app-receiver',
  standalone: true,
  imports: [FormsModule, CommonModule],
  templateUrl: './receiver.component.html',
  styleUrl: './receiver.component.css'
})
export class ReceiverComponent implements OnInit {
  relayAddress: string = '';
  relayPort?: number;
  transferName: string = '';
  isRelayServerSet = false;
  isUsingShuttle = false;
  isReceiving = false;
  receiveSuccess = false;
  constructor(private tauriService: TauriService, private router: Router, private storage: StorageService) {}
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
  reset() {
    this.isReceiving = false;
    this.receiveSuccess = false;
    this.transferName = '';
  }
  getRelayURL(): string {
    if(!this.isUsingShuttle) {
    return `ws://${this.relayAddress}:${this.relayPort}`;
    } else {
      return `${this.relayAddress}`
    } 
  }
  receiveData() {
    const relay = this.getRelayURL();
    if (this.transferName.length > 0) {
      this.isReceiving = true;
      this.receiveSuccess = false;
      this.tauriService.receive(relay, this.transferName)
        .then(sendDataReturn => {
          console.log(sendDataReturn + ' Data received successfully')
          this.receiveSuccess = true;
          setTimeout(() => {
            this.reset();
          }, 5000);
        })
        .catch(error => console.error('Error receiving data:', error));
    } else {
      console.error('No files to receive.');
    }
  }
}
