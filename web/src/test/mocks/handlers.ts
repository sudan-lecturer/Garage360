import { http, HttpResponse } from 'msw';

export const handlers = [
  http.post('/api/v1/auth/login', async ({ request }) => {
    const body = await request.json() as { email: string; password: string };
    
    if (body.email === 'fail@test.com') {
      return HttpResponse.json(
        { error: { message: 'Invalid credentials' } },
        { status: 401 }
      );
    }

    return HttpResponse.json({
      access_token: 'mock-access-token',
      refresh_token: 'mock-refresh-token',
      token_type: 'Bearer',
      expires_in: 3600,
      user: {
        id: 'user-1',
        email: body.email,
        name: 'Test User',
        role: 'admin',
        tenant_id: 'tenant-1',
      },
    });
  }),

  http.post('/api/v1/auth/refresh', async ({ request }) => {
    const body = await request.json() as { refresh_token: string };
    
    if (body.refresh_token === 'expired-token') {
      return HttpResponse.json(
        { error: { message: 'Token expired' } },
        { status: 401 }
      );
    }

    return HttpResponse.json({
      access_token: 'new-access-token',
      refresh_token: 'new-refresh-token',
      token_type: 'Bearer',
      expires_in: 3600,
    });
  }),

  http.get('/api/v1/auth/me', () => {
    return HttpResponse.json({
      id: 'user-1',
      email: 'test@example.com',
      name: 'Test User',
      role: 'admin',
      tenant_id: 'tenant-1',
    });
  }),
];
