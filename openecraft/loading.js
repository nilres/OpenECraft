// The progress of the download of the module on the loading screen (index.html):
// trunk calls these functions while it fetches the WebAssembly (data-initializer).
// `total` is the size of the file as built, so the bar is right for a compressed
// transfer too.
export default function () {
    const bar = document.getElementById("loading-bar");
    const amount = document.getElementById("loading-amount");
    const megabytes = bytes => (bytes / 1048576).toFixed(1);
    return {
        onStart: () => {},
        onProgress: ({ current, total }) => {
            if (!bar || !amount || !(total > 0)) return;
            const part = Math.min(current / total, 1);
            bar.style.width = (part * 100).toFixed(1) + "%";
            amount.textContent = `${Math.round(part * 100)} % (${megabytes(Math.min(current, total))} / ${megabytes(total)} MB)`;
        },
        onComplete: () => {},
        onSuccess: () => {},
        onFailure: () => {
            const loading = document.getElementById("loading");
            if (loading) loading.textContent = "OpenECraft failed to load. See the browser console for details.";
        },
    };
}
