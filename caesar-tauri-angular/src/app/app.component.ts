import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ActivatedRoute, Router, RouterOutlet } from '@angular/router';
import { SenderComponent } from './components/sender/sender.component';
import { TauriService } from './services/tauri.service';
import { ReceiverComponent } from './components/receiver/receiver.component';
import { SettingsMenuComponent } from './components/settings-menu/settings-menu.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, RouterOutlet, SenderComponent, ReceiverComponent, SettingsMenuComponent],
  templateUrl: './app.component.html',
  styleUrl: './app.component.css'
})
export class AppComponent implements OnInit {
  constructor(private tauriService: TauriService, private route: ActivatedRoute) {}
  ngOnInit() {
    console.log("Init")
    // this.tauriService.serve(8000, 'localhost').then(message => console.log(message));
  }
}
