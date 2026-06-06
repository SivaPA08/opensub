<script lang="ts">
    export let text: string = "";
    export let speed: number = 1;
    export let enableShadows: boolean = true;
    export let enableOnHover: boolean = true;
    export let className: string = "";

    $: inlineStyles = `
        --after-duration: ${speed * 3}s;
        --before-duration: ${speed * 2}s;
        --after-shadow: ${enableShadows ? "-5px 0 red" : "none"};
        --before-shadow: ${enableShadows ? "5px 0 cyan" : "none"};
    `;

    $: hoverClass = enableOnHover ? "enable-on-hover" : "";
</script>

<div
    class={`glitch ${hoverClass} ${className}`.trim()}
    style={inlineStyles}
    data-text={text}
>
    {text}
</div>

<style>
    .glitch {
        color: inherit;
        font-size: inherit;
        font-family: inherit;
        font-weight: inherit;
        white-space: pre-wrap;
        word-break: break-word;
        position: relative;
        margin: 0 auto;
        user-select: none;
        cursor: pointer;
        display: inline-block;
        width: 100%;
        box-sizing: border-box;
    }

    .glitch::after,
    .glitch::before {
        content: attr(data-text);
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        color: inherit;
        background: inherit;
        font-family: inherit;
        font-weight: inherit;
        white-space: pre-wrap;
        word-break: break-word;
        overflow: hidden;
        clip-path: inset(0 0 0 0);
        pointer-events: none;
        box-sizing: border-box;
    }

    .glitch:not(.enable-on-hover)::after {
        left: 10px;
        text-shadow: var(--after-shadow, -10px 0 red);
        animation: animate-glitch var(--after-duration, 3s) infinite linear
            alternate-reverse;
    }

    .glitch:not(.enable-on-hover)::before {
        left: -10px;
        text-shadow: var(--before-shadow, 10px 0 cyan);
        animation: animate-glitch var(--before-duration, 2s) infinite linear
            alternate-reverse;
    }

    .glitch.enable-on-hover::after,
    .glitch.enable-on-hover::before {
        content: "";
        opacity: 0;
        animation: none;
    }

    .glitch.enable-on-hover:hover::after {
        content: attr(data-text);
        opacity: 1;
        left: 10px;
        text-shadow: var(--after-shadow, -10px 0 red);
        animation: animate-glitch var(--after-duration, 3s) infinite linear
            alternate-reverse;
    }

    .glitch.enable-on-hover:hover::before {
        content: attr(data-text);
        opacity: 1;
        left: -10px;
        text-shadow: var(--before-shadow, 10px 0 cyan);
        animation: animate-glitch var(--before-duration, 2s) infinite linear
            alternate-reverse;
    }

    @keyframes animate-glitch {
        0% {
            clip-path: inset(20% 0 50% 0);
        }
        5% {
            clip-path: inset(10% 0 60% 0);
        }
        10% {
            clip-path: inset(15% 0 55% 0);
        }
        15% {
            clip-path: inset(25% 0 35% 0);
        }
        20% {
            clip-path: inset(30% 0 40% 0);
        }
        25% {
            clip-path: inset(40% 0 20% 0);
        }
        30% {
            clip-path: inset(10% 0 60% 0);
        }
        35% {
            clip-path: inset(15% 0 55% 0);
        }
        40% {
            clip-path: inset(25% 0 35% 0);
        }
        45% {
            clip-path: inset(30% 0 40% 0);
        }
        50% {
            clip-path: inset(20% 0 50% 0);
        }
        55% {
            clip-path: inset(10% 0 60% 0);
        }
        60% {
            clip-path: inset(15% 0 55% 0);
        }
        65% {
            clip-path: inset(25% 0 35% 0);
        }
        70% {
            clip-path: inset(30% 0 40% 0);
        }
        75% {
            clip-path: inset(40% 0 20% 0);
        }
        80% {
            clip-path: inset(20% 0 50% 0);
        }
        85% {
            clip-path: inset(10% 0 60% 0);
        }
        90% {
            clip-path: inset(15% 0 55% 0);
        }
        95% {
            clip-path: inset(25% 0 35% 0);
        }
        100% {
            clip-path: inset(30% 0 40% 0);
        }
    }
</style>
