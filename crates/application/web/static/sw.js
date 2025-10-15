self.addEventListener('install', (event) => {
    console.log('Service Worker installing.');
    // Cache assets here if needed
});

self.addEventListener('fetch', (event) => {
    event.respondWith(fetch(event.request).catch(() => {
        return caches.match('/static/html/offline.html');
    }));
});
