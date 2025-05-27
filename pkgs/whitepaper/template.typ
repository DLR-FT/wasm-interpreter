#import "@preview/ccicons:1.0.0": cc-by-sa

#let setup_template(title: [], author: [], keywords: (), affiliation: [], contents) = {
  set document(title: title, author: author, keywords: keywords)
  set page(
    paper: "a4", columns: 1, header: context{
      if counter(page).get().first() > 1 {
        align(right, title)
      }
    }, footer: context{
      set text(8pt)
      let left_footer = [
        License: #link("https://creativecommons.org/licenses/by-sa/4.0/")[CC-BY-SA #cc-by-sa] \
        Copyright © 2024-#datetime.today().year() German Aerospace Center (DLR). All
        rights reserved.
      ]
      let right_footer = counter(page).display("1 of 1", both: true)

      grid(columns: (1fr, auto), align: (left, right), left_footer, right_footer)
    },
  )

  // Style
  set heading(numbering: "1.")

  align(center, text(17pt)[*#title*])

  grid(columns: (1fr), align(center)[
    #author \
    #affiliation
  ])

  contents
}
