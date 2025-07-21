// Semantic version compare utility
function semverCompare(a, b) {
    const parse = v => v.replace(/^v/, '').split(/[\.-]/).map((x, i) => i < 3 ? parseInt(x, 10) : x);
    const isPre = v => typeof v[3] !== 'undefined';

    const av = parse(a);
    const bv = parse(b);

    for (let i = 0; i < 3; i++) {
        if ((av[i] || 0) !== (bv[i] || 0)) return (bv[i] || 0) - (av[i] || 0);
    }

    // Non-pre-release wins over pre-release
    if (isPre(av) && !isPre(bv)) return 1;
    if (!isPre(av) && isPre(bv)) return -1;

    // If both are pre-releases, compare their pre-release tags
    if (isPre(av) && isPre(bv)) return av[3].localeCompare(bv[3]);

    return 0;
}

document.addEventListener("DOMContentLoaded", () => {
    // Get the base URL from the current location path
    const baseUrl = document.location.pathname.split('/').slice(0, -2).join('/');

    // Utility function to create and populate the version selector
    const createVersionSelector = (versions) => {
        const versionSelector = document.createElement("select");
        versionSelector.id = "version-selector";

        // Sort and iterate through the versions to populate the selector
        Object.entries(versions)
            .sort(([a], [b]) => {
                if (a === "latest") return -1;
                if (b === "latest") return 1;
                return semverCompare(a, b);
            })
            .forEach(([name, url]) => {
                const option = document.createElement("option");
                option.value = `${baseUrl}${url}`;
                option.textContent = name;
                // Pre-select the matching version
                option.selected = name === window.location.pathname.split('/')[2];
                versionSelector.appendChild(option);
            });

        // Redirect to the selected version when changed
        versionSelector.addEventListener("change", () => {
            window.location.href = versionSelector.value;
        });

        return versionSelector;
    };

    // Fetch the versions.json file and initialize the selector
    fetch(`${baseUrl}/versions.json`)
        .then(response => response.ok ? response.json() : Promise.reject(`Error: ${response.statusText}`))
        .then(versions => {
            // Locate the navigation element to append the version selector
            const nav = document.querySelector(".right-buttons");
            if (!nav) return console.error(".right-buttons element not found.");

            const versionBox = document.createElement("div");
            versionBox.id = "version-box";
            versionBox.appendChild(createVersionSelector(versions));
            nav.appendChild(versionBox);
        })
        .catch(error => console.error("Failed to fetch versions.json:", error));
});
