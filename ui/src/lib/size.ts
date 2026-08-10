const MEGABYTE = 1024 * 1024;

export const megabytes = (bytes: number): number => Math.max(1, Math.round(bytes / MEGABYTE));
