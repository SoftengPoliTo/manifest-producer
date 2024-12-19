function filterTable() {
    const input = document.getElementById("searchInput");
    const filter = input.value.toLowerCase();
    const table = document.getElementById("functionsTable");
    const tr = table.getElementsByTagName("tr");

    for (let i = 1; i < tr.length; i++) {
        const td = tr[i].getElementsByTagName("td")[0];
        if (td) {
            const txtValue = td.textContent || td.innerText;
            tr[i].style.display = txtValue.toLowerCase().indexOf(filter) > -1 ? "" : "none";
        }
    }
}

let sortOrder = {};
function sortTable(colIndex, type = 'str') {
    const table = document.getElementById("functionsTable");
    const tbody = table.querySelector("tbody");
    const rows = Array.from(tbody.querySelectorAll("tr"));

    sortOrder[colIndex] = sortOrder[colIndex] === 'asc' ? 'desc' : 'asc';

    const sortedRows = rows.sort((a, b) => {
        const aText = a.querySelectorAll("td")[colIndex].textContent.trim();
        const bText = b.querySelectorAll("td")[colIndex].textContent.trim();

        let comparison = 0;

        if (type === 'num') {
            comparison = parseFloat(aText.replace(/[^0-9.]/g, "")) - parseFloat(bText.replace(/[^0-9.]/g, ""));
        } else {
            comparison = aText.localeCompare(bText);
        }

        return sortOrder[colIndex] === 'asc' ? comparison : -comparison;
    });

    tbody.innerHTML = "";
    sortedRows.forEach(row => tbody.appendChild(row));
}

function copyToClipboard(functionName) {
    const codeBlock = document.querySelector(`#code-${functionName}`);
    if (!codeBlock) {
        console.error(`Code block not found for function: ${functionName}`);
        return;
    }
    const codeText = codeBlock.innerText;

    navigator.clipboard.writeText(codeText).then(() => {
        const copyButton = document.querySelector(`#copy-btn-${functionName}`);
        copyButton.classList.add('is-success');
        copyButton.innerHTML = '<span class="icon"><i class="fas fa-check"></i></span>';
        setTimeout(() => {
            copyButton.classList.remove('is-success');
            copyButton.innerHTML = '<span class="icon"><i class="fas fa-copy"></i></span>';
        }, 2000);
    });
}

function filterFunctions() {
    const searchInput = document.getElementById('searchInput').value.toLowerCase();
    const functionSections = document.querySelectorAll('.function-section');
    functionSections.forEach(section => {
        const functionName = section.querySelector('.collapsible').textContent.toLowerCase();
        if (functionName.includes(searchInput)) {
            section.classList.remove('hidden');
        } else {
            section.classList.add('hidden');
        }
    });
}

document.addEventListener('DOMContentLoaded', () => {
    const collapsibleButtons = document.querySelectorAll('.collapsible');
    collapsibleButtons.forEach(button => {
        button.addEventListener('click', () => {
            const content = button.nextElementSibling;
            content.style.display = content.style.display === 'block' ? 'none' : 'block';
            button.classList.toggle('is-active');
        });
    });
});

function scrollToTop() {
    window.scrollTo({
        top: 0,
        behavior: 'smooth'
    });
}

function scrollToBottom() {
    window.scrollTo({
        top: document.body.scrollHeight,
        behavior: 'smooth'
    });
}
