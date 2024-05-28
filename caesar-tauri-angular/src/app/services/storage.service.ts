import { Injectable } from '@angular/core';

@Injectable({
  providedIn: 'root'
})
export class StorageService {

	// ----------------- Session ------------------------
  // Ruft einen Wert aus dem sessionStorage ab
  getSessionEntry(key: string): any {
    const item = sessionStorage.getItem(key);
    return item ? JSON.parse(item) : null;
  }

	public setSessionEntry(key: string, data: unknown): void {
		sessionStorage.setItem(key, JSON.stringify(data));
	}

	public deleteSessionEntry(key: string): void {
		sessionStorage.removeItem(key);
	}

	public clearSession(): void {
		sessionStorage.clear();
	}

	public isSessionEmpty(): boolean {
		return sessionStorage.length === 0;
	}

	// ----------------- Local ------------------------
	public setLocalEntry(key: string, data: unknown): void {
		localStorage.setItem(key, JSON.stringify(data));
	}

	public getLocalEntry(key: string): any {
    const item = localStorage.getItem(key);
    return item ? JSON.parse(item) : null;
  }

	public deleteLocalEntry(key: string): void {
		localStorage.removeItem(key);
	}

	public clearLocal(): void {
		localStorage.clear();
	}

	public isLocalEmpty(): boolean {
		return localStorage.length === 0;
	}

	// ----------------- General ------------------------
	public clearAll(): void {
		this.clearLocal();
		this.clearSession();
	}
}