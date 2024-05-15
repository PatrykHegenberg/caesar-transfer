import { Routes } from "@angular/router";
import { SenderComponent } from "./components/sender/sender.component";
import { ReceiverComponent } from "./receiver/receiver.component";
import { HomeComponent } from "./home/home.component";

export const routes: Routes = [
    { path: 'sender', component: SenderComponent },
    { path: 'receiver', component: ReceiverComponent },
    { path: '', component: HomeComponent }
];
