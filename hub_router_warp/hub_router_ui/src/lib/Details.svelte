<script lang="ts">
    import { breadcrumb } from "./data";

</script>

<div class="details-wrapper" style="--theme-depth: {$breadcrumb.length - 1}">

</div>

<style>
    .details-wrapper {
        flex-grow: 1;
        background-color: hsl(250, 25%, calc(10% + 2% * (var(--theme-depth) + 1)));
    }
</style>