import { describe, it, expect, beforeEach } from 'vitest';
import i18n from './index';

describe('i18n Configuration', () => {
  beforeEach(() => {
    i18n.changeLanguage('en');
  });

  describe('Initialization', () => {
    it('should be initialized', () => {
      expect(i18n.isInitialized).toBe(true);
    });

    it('should have default language set to en', () => {
      expect(i18n.language).toBe('en');
    });
  });

  describe('Common Translations', () => {
    it('should have loading translation', () => {
      expect(i18n.t('common.loading')).toBe('Loading...');
    });

    it('should have save translation', () => {
      expect(i18n.t('common.save')).toBe('Save');
    });

    it('should have cancel translation', () => {
      expect(i18n.t('common.cancel')).toBe('Cancel');
    });

    it('should have delete translation', () => {
      expect(i18n.t('common.delete')).toBe('Delete');
    });

    it('should have edit translation', () => {
      expect(i18n.t('common.edit')).toBe('Edit');
    });

    it('should have create translation', () => {
      expect(i18n.t('common.create')).toBe('Create');
    });

    it('should have search translation', () => {
      expect(i18n.t('common.search')).toBe('Search');
    });

    it('should have filter translation', () => {
      expect(i18n.t('common.filter')).toBe('Filter');
    });

    it('should have export translation', () => {
      expect(i18n.t('common.export')).toBe('Export');
    });

    it('should have import translation', () => {
      expect(i18n.t('common.import')).toBe('Import');
    });
  });

  describe('Auth Translations', () => {
    it('should have login translation', () => {
      expect(i18n.t('auth.login')).toBe('Sign In');
    });

    it('should have logout translation', () => {
      expect(i18n.t('auth.logout')).toBe('Sign Out');
    });

    it('should have email translation', () => {
      expect(i18n.t('auth.email')).toBe('Email');
    });

    it('should have password translation', () => {
      expect(i18n.t('auth.password')).toBe('Password');
    });

    it('should have forgot password translation', () => {
      expect(i18n.t('auth.forgotPassword')).toBe('Forgot password?');
    });
  });

  describe('Navigation Translations', () => {
    it('should have dashboard translation', () => {
      expect(i18n.t('nav.dashboard')).toBe('Dashboard');
    });

    it('should have customers translation', () => {
      expect(i18n.t('nav.customers')).toBe('Customers');
    });

    it('should have vehicles translation', () => {
      expect(i18n.t('nav.vehicles')).toBe('Vehicles');
    });

    it('should have jobs translation', () => {
      expect(i18n.t('nav.jobs')).toBe('Jobs');
    });

    it('should have inventory translation', () => {
      expect(i18n.t('nav.inventory')).toBe('Inventory');
    });

    it('should have purchases translation', () => {
      expect(i18n.t('nav.purchases')).toBe('Purchases');
    });

    it('should have billing translation', () => {
      expect(i18n.t('nav.billing')).toBe('Billing');
    });

    it('should have dvi translation', () => {
      expect(i18n.t('nav.dvi')).toBe('DVI');
    });

    it('should have assets translation', () => {
      expect(i18n.t('nav.assets')).toBe('Assets');
    });

    it('should have hr translation', () => {
      expect(i18n.t('nav.hr')).toBe('HR');
    });

    it('should have reports translation', () => {
      expect(i18n.t('nav.reports')).toBe('Reports');
    });

    it('should have settings translation', () => {
      expect(i18n.t('nav.settings')).toBe('Settings');
    });
  });

  describe('Interpolation', () => {
    it('should not escape values by default', () => {
      expect(i18n.options.interpolation?.escapeValue).toBe(false);
    });
  });
});
