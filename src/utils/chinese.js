import { ConverterBuilder } from 'opencc-js/core';
import * as Locale from 'opencc-js/preset/cn2t';

const convertSimplifiedToTraditional = ConverterBuilder(Locale)({ from: 'cn', to: 'twp' });

export function simplifiedToTraditional(text) {
    return convertSimplifiedToTraditional(text);
}
