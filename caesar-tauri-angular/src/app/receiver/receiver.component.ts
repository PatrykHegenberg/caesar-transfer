import { Component } from '@angular/core';
import { TauriService } from '../services/tauri.service';
import { Router } from '@angular/router';

@Component({
  selector: 'app-receiver',
  standalone: true,
  imports: [],
  templateUrl: './receiver.component.html',
  styleUrl: './receiver.component.css'
})
export class ReceiverComponent {
  constructor(private tauriService: TauriService, private router: Router) {}
  redirectToHome() {
    this.router.navigate([''])
  }
}
