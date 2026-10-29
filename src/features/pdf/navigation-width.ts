export const navigationWidthKey='researchclip.pdf.navigation-width.v1';
export function navigationWidth(value:number,available=1600){if(!Number.isFinite(value)||!Number.isFinite(available))throw Error('PDF导航宽度无效。');return Math.round(Math.max(180,Math.min(value,800,Math.max(180,available*.65))));}
