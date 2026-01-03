import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    loadComponent: () => import('./pages/home/home.component').then(m => m.HomeComponent)
  },
  {
    path: 'applets',
    loadComponent: () => import('./pages/applets/applets.component').then(m => m.AppletsComponent)
  },
  {
    path: 'building',
    loadComponent: () => import('./pages/building/building.component').then(m => m.BuildingComponent)
  },
  {
    path: 'comparison',
    loadComponent: () => import('./pages/comparison/comparison.component').then(m => m.ComparisonComponent)
  },
  {
    path: 'benchmarks',
    loadComponent: () => import('./pages/benchmarks/benchmarks.component').then(m => m.BenchmarksComponent)
  },
  {
    path: 'api',
    loadComponent: () => import('./pages/api/api.component').then(m => m.ApiComponent)
  },
  {
    path: '**',
    redirectTo: ''
  }
];
