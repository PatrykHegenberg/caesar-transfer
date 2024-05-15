import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router, RouterOutlet } from '@angular/router';
import { SenderComponent } from './components/sender/sender.component';
import { TauriService } from './services/tauri.service';
import { ReceiverComponent } from './receiver/receiver.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, SenderComponent, ReceiverComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})
export class AppComponent implements OnInit {
  constructor(private tauriService: TauriService, private router: Router) {}
  ngOnInit() {
    console.log("Init")
    // this.tauriService.serve(8000, 'localhost').then(message => console.log(message));
  }
}
