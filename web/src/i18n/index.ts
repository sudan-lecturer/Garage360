import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

i18n.use(initReactI18next).init({
  resources: {
    en: {
      translation: {
        common: {
          loading: 'Loading...',
          save: 'Save',
          cancel: 'Cancel',
          delete: 'Delete',
          edit: 'Edit',
          create: 'Create',
          search: 'Search',
          filter: 'Filter',
          export: 'Export',
          import: 'Import',
        },
        auth: {
          login: 'Sign In',
          logout: 'Sign Out',
          email: 'Email',
          password: 'Password',
          forgotPassword: 'Forgot password?',
        },
        nav: {
          dashboard: 'Dashboard',
          customers: 'Customers',
          vehicles: 'Vehicles',
          jobs: 'Jobs',
          inventory: 'Inventory',
          purchases: 'Purchases',
          billing: 'Billing',
          dvi: 'DVI',
          assets: 'Assets',
          hr: 'HR',
          reports: 'Reports',
          settings: 'Settings',
        },
      },
    },
  },
  lng: 'en',
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
});

export default i18n;
