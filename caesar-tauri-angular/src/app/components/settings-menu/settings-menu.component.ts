import { CommonModule } from '@angular/common';
import { Component, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { StorageService } from '../../services/storage.service';
import { Router } from '@angular/router';

@Component({
  selector: 'app-settings-menu',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './settings-menu.component.html',
  styleUrl: './settings-menu.component.css'
})
export class SettingsMenuComponent implements OnInit {
  constructor(private storage: StorageService, private router: Router) {}
  menuVisible = false;
  settings = {
    relayServer: '',
    port: 0,
  };

  ngOnInit(): void {
      if(this.storage.getLocalEntry('relayServer')) {
        this.settings.relayServer = this.storage.getLocalEntry('relayServer');
      }
      if(this.storage.getLocalEntry('port')) {
        this.settings.port = this.storage.getLocalEntry('port');
      }
  }

  toggleMenu(): void {
    this.menuVisible = !this.menuVisible;
  }

  saveSettings(): void {
    this.storage.setLocalEntry('relayServer', this.settings.relayServer)
    this.storage.setLocalEntry('port', this.settings.port)
    alert("The settings have been saved!");
    this.menuVisible = !this.menuVisible;
  }

  resetSettings(): void {
    this.settings.relayServer = '';
    this.settings.port = 0;
    this.storage.clearLocal();
    alert("The settings have been reset!")
    this.menuVisible = !this.menuVisible;
  }
}