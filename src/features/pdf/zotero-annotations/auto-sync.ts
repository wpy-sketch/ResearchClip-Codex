import type {WriteInput} from './Writer';
import {validLocalInput} from './sync-model';
export function completeLocalChange(input:WriteInput|null):boolean {
 if(!input||!validLocalInput(input))return false;
 if(input.action==='delete'||input.annotationType==='image')return true;
 return ['highlight','underline'].includes(input.annotationType)?!!input.text.trim():!!input.comment.trim();
}
