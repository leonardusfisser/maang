const csrf = 'TOKEN_FROM_LOGIN_RESPONSE';
htmx.on('htmx:configRequest', (e) => {
    e.detail.headers['X-CSRF-Token'] = csrf;
});