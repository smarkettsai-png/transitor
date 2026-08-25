export function getSelectedText(documentObject = document) {
    const activeElement = documentObject?.activeElement;
    if (
        activeElement &&
        typeof activeElement.value === 'string' &&
        Number.isInteger(activeElement.selectionStart) &&
        Number.isInteger(activeElement.selectionEnd) &&
        activeElement.selectionEnd > activeElement.selectionStart
    ) {
        return activeElement.value.slice(activeElement.selectionStart, activeElement.selectionEnd);
    }

    const selection = documentObject?.getSelection?.();
    return selection?.toString?.() ?? '';
}
