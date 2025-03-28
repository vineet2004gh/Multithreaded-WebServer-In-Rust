let threadIndex = 0;

function simulateRequest() {
    const threads = document.querySelectorAll('.thread-box');
    const clientBox = document.getElementById('client-box');
    const serverBox = document.getElementById('server-box');
    
    const request = document.createElement('div');
    request.classList.add('request');
    request.textContent = 'R';
    document.body.appendChild(request);

    const clientRect = clientBox.getBoundingClientRect();
    request.style.left = `${clientRect.left + window.scrollX + clientRect.width / 2 - 12}px`;
    request.style.top = `${clientRect.top + window.scrollY + clientRect.height / 2 - 12}px`;

    setTimeout(() => {
        const serverRect = serverBox.getBoundingClientRect();
        request.style.left = `${serverRect.left + window.scrollX + serverRect.width / 2 - 12}px`;
        request.style.top = `${serverRect.top + window.scrollY + serverRect.height / 2 - 12}px`;

        setTimeout(() => {
            const targetThread = threads[threadIndex];
            const threadRect = targetThread.getBoundingClientRect();
            
            request.style.left = `${threadRect.left + window.scrollX + threadRect.width / 2 - 12}px`;
            request.style.top = `${threadRect.top + window.scrollY + threadRect.height / 2 - 12}px`;

            setTimeout(() => request.remove(), 1500);
            threadIndex = (threadIndex + 1) % threads.length;
        }, 1000);
    }, 1000);
}

function simulateProxyRequest() {
    const clientBox = document.getElementById('proxy-client');
    const proxyBox = document.getElementById('proxy-box');
    const mainServer = document.getElementById('main-server');
    
    const requestP = document.createElement('div');
    requestP.classList.add('blue-circle');
    requestP.textContent = ''; 
    document.body.appendChild(requestP);

    const clientRect = clientBox.getBoundingClientRect();
    requestP.style.left = `${clientRect.left + window.scrollX + clientRect.width / 2 }px`;
    requestP.style.top = `${clientRect.top + window.scrollY + clientRect.height / 2 + 80}px`;

    // Move to Proxy Server (diagonal)
    setTimeout(() => {
        const proxyRect = proxyBox.getBoundingClientRect();
        requestP.style.transition = 'left 1s, top 1s'; // Smooth transition
        requestP.style.left = `${proxyRect.left + window.scrollX }px`; // Move to left edge
        requestP.style.top = `${proxyRect.top + window.scrollY + proxyRect.height / 2 - 12}px`; // Center vertically
    
        // Step 2: Move from left edge to right edge of Proxy Server
        setTimeout(() => {
            requestP.style.left = `${proxyRect.right + window.scrollX - 12}px`; // Move to right edge
            requestP.style.top = `${proxyRect.top + window.scrollY + proxyRect.height / 2 - 12}px`; // Center vertically
    
            // Step 3: Move to Main Server (left edge)
            setTimeout(() => {
                const serverRect = mainServer.getBoundingClientRect();
                requestP.style.left = `${serverRect.left + window.scrollX + serverRect.height / 2 +2}px`; // Move to left edge of main server
                requestP.style.top = `${serverRect.top + window.scrollY + serverRect.height / 2 + 40}px`; // Center vertically
    
                // Remove requestP after a delay
                setTimeout(() => requestP.remove(), 1500);
            }, 1000); // Adjust the delay if needed
        }, 500); // Adjust the delay if needed
    }, 1000); 
}
