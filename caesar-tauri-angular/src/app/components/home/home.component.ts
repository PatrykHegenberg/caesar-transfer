import { Component } from '@angular/core';
import { Router } from '@angular/router';
import { SettingsMenuComponent } from '../settings-menu/settings-menu.component';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [SettingsMenuComponent],
  templateUrl: './home.component.html',
  styleUrl: './home.component.css'
})
export class HomeComponent {
  constructor(private router: Router) {}
  redirectToSender() {
    this.router.navigate(['/sender'])
  }

  redirectToReceiver() {
    this.router.navigate(['/receiver'])
  }
}
