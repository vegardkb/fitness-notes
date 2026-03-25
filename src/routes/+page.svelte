<script>
    const date = new Date();
    let route = "day/" + date.toISOString().split("T")[0];
</script>

<main>
    <h1>Fitness Notes!</h1>
    <a href={route}>To current day</a>
</main>
