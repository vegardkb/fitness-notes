import { browser, expect, $, $$ } from '@wdio/globals'

// End-to-end flow against the real app: log a set for an exercise, then open
// a historical session from the exercise history tab and verify its sets.
//
// The test binary is built with `pnpm build:wdio` (wdio cargo feature +
// separate app identifier). On every boot it wipes the database and inserts
// one fixed workout — Bench Press (exercise id 1) with sets 80x5, 90x5,
// 100x5, two days ago (workout_exercise id 1) — so ids and data are
// deterministic on any machine and your real database is never touched.

// The app computes dates in local time (lib/date.ts todayStr) — mirror that.
function localDateStr(): string {
    const d = new Date()
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

describe('log a set, then open the session from history', () => {
  it('adds a set today and shows the seeded session from history', async () => {
    // the webview is already on the home feed when the session starts —
    // no browser.url() call (it hangs on Tauri custom protocols)
    const today = localDateStr()
    const dayCard = $(`#day-${today}`)
    await dayCard.waitForDisplayed({ timeout: 15000 })

    // today's card is empty: open the exercise picker via the dumbbell icon
    // (lucide icons render as svg.lucide-dumbbell)
    await $(`#day-${today} button:has(svg.lucide-dumbbell)`).click()

    // picker flow: Select category → Chest → Bench Press
    await expect($('h1')).toHaveText('Select category')
    await $('span=Chest').click()
    await $('span=Bench Press').click()

    // selecting an exercise creates today's workout_exercise and opens
    // the sets page — fresh DB, so it is the second one (id 3)
    await expect(browser).toHaveUrl(
      expect.stringMatching(/\/exercise\/\d+\/3$/),
    )
    await expect($('p.empty')).toHaveText(
      'No sets yet. Add your first set below.',
    )

    // the form pre-fills the last logged set (100x5 from the seed) —
    // enter explicit values instead so the assertion is unambiguous
    let inputs = await $$('input[type="number"]')
    await inputs[0].setValue(60) // weight
    await inputs[1].setValue(8) // reps
    await $('.add-btn').click()

    // the new set appears in the list (formatWeight renders "60.0")
    await expect($('.list .list-item .stat-val--weight')).toHaveText(
      '60.0',
    )
    await expect($('.list .list-item .stat-val--reps')).toHaveText('8')

    // ExerciseHeader tabs are anchors with aria-labels
    await $('[aria-label="History"]').click()
    await expect($('p')).toHaveText('History')

    // two sessions exist: today's and the seeded one from 2 days ago,
    // sorted newest first — open the older one
    await expect($$('.exercise-card-header')).toBeElementsArrayOfSize(2)
    let cards = await $$('.exercise-card-header')
    await cards[1].click()

    // navigated to the seeded workout_exercise (id 1) and its sets are
    // rendered on the sets page
    await expect(browser).toHaveUrl(
      expect.stringMatching(/\/exercise\/\d+\/1$/),
    )
    await expect($('.history-header h1')).toHaveText('Bench Press')

    let setRows = await $$('.list .list-item')
    expect(setRows.length).toBe(3)
    let weights: string[] = []
    for (const w of await $$('.list .stat-val--weight')) {
      weights.push(await w.getText())
    }
    expect(weights).toEqual(['80.0', '90.0', '100.0'])
    for (const r of await $$('.list .stat-val--reps')) {
      await expect(r).toHaveText('5')
    }

    await $(`[aria-label="Back"]`).click()

    // today's card is empty: open the exercise picker via the dumbbell icon
    // (lucide icons render as svg.lucide-dumbbell)
    await $(`#day-${today} button:has(svg.lucide-dumbbell)`).click()

    // picker flow: Select category → Chest → Bench Press
    await expect($('h1')).toHaveText('Select category')
    await $('span=Chest').click()
    await $('span=DB Bench Press').click()

    // selecting an exercise creates today's workout_exercise and opens
    // the sets page — fresh DB, so it is the fourth one (id 4)
    await expect(browser).toHaveUrl(
      expect.stringMatching(/\/exercise\/\d+\/4$/),
    )
    await expect($('p.empty')).toHaveText(
      'No sets yet. Add your first set below.',
    )

    // the form pre-fills the last logged set (100x5 from the seed) —
    // enter explicit values instead so the assertion is unambiguous
    inputs = await $$('input[type="number"]')
    await inputs[0].setValue(60) // weight
    await inputs[1].setValue(8) // reps
    await $('.add-btn').click()

    // the new set appears in the list (formatWeight renders "60.0")
    await expect($('.list .list-item .stat-val--weight')).toHaveText(
      '60.0',
    )
    await expect($('.list .list-item .stat-val--reps')).toHaveText('8')

    // ExerciseHeader tabs are anchors with aria-labels
    await $('[aria-label="History"]').click()
    await expect($('p')).toHaveText('History')

    // two sessions exist: today's and the seeded one from 2 days ago,
    // sorted newest first — open the older one
    await expect($$('.exercise-card-header')).toBeElementsArrayOfSize(2)
    cards = await $$('.exercise-card-header')
    await cards[1].click()

    // navigated to the seeded workout_exercise (id 2) and its sets are
    // rendered on the sets page
    await expect(browser).toHaveUrl(
      expect.stringMatching(/\/exercise\/\d+\/2$/),
    )
    await expect($('.history-header h1')).toHaveText('DB Bench Press')

    setRows = await $$('.list .list-item')
    expect(setRows.length).toBe(2)
    weights = []
    for (const w of await $$('.list .stat-val--weight')) {
      weights.push(await w.getText())
    }
    expect(weights).toEqual(['40.0', '50.0'])
    for (const r of await $$('.list .stat-val--reps')) {
      await expect(r).toHaveText('8')
    }
  })
})
