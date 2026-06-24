/**
 * @param   table {HTMLTableElement}
 * @returns {string} - CSV string
 */
function getCSVString(table) {
  let csvString = "";

  for (const tr of table.rows) {
    const file = tr.cells[0].childNodes[0].textContent.trim();
    const sha1Sum = tr.cells[1].textContent.trim();
    csvString = `${csvString}${file},${sha1Sum}\n`;
  }

  return csvString;
}

/**
 * @returns {HTMLTableElement}
 */
function getTableEl() {
  const selector = "table#DataTables_Table_0";
  return document.querySelector(selector);
}

/**
 * @returns {HTMLLinkElement}
 */
function getTitleEl() {
  const selector = "a[data-appid].app";
  return document.querySelector(selector);
}

/**
 * @returns {HTMLTableElement}
 */
function getMetaTableEl() {
  const selector = "table";
  return document.querySelector(selector);
}

function genCsvFileName() {
  let title = `${getTitleEl().textContent.replaceAll(" ", "_")}`;
  let depotId = "";
  let buildId = "";

  for (const tr of getMetaTableEl().rows) {
    const key = tr.cells[0].textContent;
    const value = tr.cells[1].textContent;

    switch (key) {
      case "Depot ID":
        depotId = value;
        break;
      case "Build ID":
        buildId = value;
    }
  }

  return `${title}-${depotId}-${buildId}`;
}

function downloadCsv() {
  const csvDataUrl = `data:text/csv;base64,${btoa(getCSVString(getTableEl()))}`;

  const a = document.createElement("a");
  a.style = "display: hidden;";
  a.href = csvDataUrl;
  a.download = genCsvFileName();

  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
}

downloadCsv();
