/** @type {import('tailwindcss').Config} */
export default {
    content: ['./src/**/*.{html,js,svelte,ts}'],
    theme: {
        extend: {
            colors: {
                // Dark mode palette
                background: {
                    primary: '#0a0a0a',
                    secondary: '#141414',
                    tertiary: '#1e1e1e'
                },
                text: {
                    primary: '#ffffff',
                    secondary: '#a0a0a0',
                    tertiary: '#707070'
                },
                accent: {
                    primary: '#3b82f6',
                    hover: '#2563eb'
                },
                state: {
                    completed: '#22c55e',
                    progress: '#f59e0b',
                    unwatched: '#6b7280',
                    error: '#ef4444'
                }
            },
            fontFamily: {
                sans: ['Inter', 'system-ui', 'sans-serif']
            }
        }
    },
    plugins: []
};