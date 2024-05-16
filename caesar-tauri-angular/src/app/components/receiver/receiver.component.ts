import { Component } from '@angular/core';
import { TauriService } from '../../services/tauri.service';
import { Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-receiver',
  standalone: true,
  imports: [FormsModule, CommonModule],
  templateUrl: './receiver.component.html',
  styleUrl: './receiver.component.css'
})
export class ReceiverComponent {
  relayAddress: string = '';
  relayPort?: number;
  transferName: string = '';
  constructor(private tauriService: TauriService, private router: Router) {}
  redirectToHome() {
    this.router.navigate([''])
  }
  getRelayURL(): string {
    return `ws://${this.relayAddress}:${this.relayPort}`;
  }
  receiveData() {
    const relay = this.getRelayURL();
    if (this.transferName.length > 0) {
      this.tauriService.receive(relay, this.transferName)
        .then(sendDataReturn => console.log(sendDataReturn + ' Data received successfully'))
        .catch(error => console.error('Error receiving data:', error));
    } else {
      console.error('No files to receive.');
    }
  }
}
