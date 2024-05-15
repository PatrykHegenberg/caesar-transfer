import { Component } from '@angular/core';
import { TauriService } from '../../services/tauri.service';
import { invoke } from "@tauri-apps/api/core";
import { Router } from '@angular/router';

@Component({
  selector: 'app-sender',
  standalone: true,
  imports: [],
  templateUrl: './sender.component.html',
  styleUrl: './sender.component.css'
})
export class SenderComponent {
  constructor(private tauriService: TauriService, private router: Router) {}
  greetingMessage = "";
  greet(event: SubmitEvent, name: string): void {
    event.preventDefault();

    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    invoke<string>("greet", { name }).then((text) => {
      this.greetingMessage = text;
    });
  }

  redirectToHome() {
    this.router.navigate([''])
  }

  sendData() {
    const relay = 'ws://[::1]:8000';
    const files = ['C:\\Projekte\\Rust\\caesar-transfer\\caesar-tauri-angular\\src\\assets\\file1.txt'];
    this.tauriService.send(relay, files)
      .then((sendDataReturn) => console.log(sendDataReturn + 'Data sent successfully'))
      .catch(error => console.error('Error sending data:', error));
  }
}