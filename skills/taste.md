---
name: taste
description: Your personal taste skill — encoding your design sensibilities, brand guidelines, aesthetic preferences, and design principles into a reusable skill reference.
---

# Taste

## What is Taste?

Taste is not randomness. It is trained sensitivity — the ability to recognize what is good, what is beautiful, and what is right when it matters. Taste is trainable.

Every time you say something is nice or something is ugly, you are applying a judgment call. Taste is nothing more than an accumulation of those judgments written down.

## Design Principles

### It should work before it looks good

No amount of polish can save a feature that is impossible to use. People will forgive bad looking things. They will not forgive things that are impossible to use.

Always prioritize usability over aesthetics. But also remember: usability and aesthetics are not separate things — good usability and good aesthetics are interdependent.

### Visible constraints

Use constraints not to restrict creativity but to define boundaries for decision-making.

- Set a maximum color palette per screen
- Define a maximum of 3-4 primary font weights on a page
- Limit your button shapes to 2 options
- Maximum 5 different border radius values across your product

Constraints reduce decision fatigue and make you more deliberate.

### Design your system before your design

Sketch your complete interaction model first. Map out every state the user can reach and how it flows. This is your design spec before you write a single line of production code.

### Negative space is a design decision

Empty space is not the absence of design. It is a deliberate design decision. When you surround something with empty space you are telling the user what is important and where they should focus attention.

### Consistency compounds

The same pattern repeated 100 times creates a rhythm users come to enjoy. Inconsistency creates noise and friction. If you use 7 different ways to say "yes" across your product you have created friction.

### Design for the future

What looks boring today will look dated in three years. What looks jarring today may feel intentional in a year. Build systems that are timeless.

### Small details aggregate

Three things that feel good individually are annoying when they don't feel good together. Your buttons shouldn't just use the same elevation. The primary button should have a subtle glow or highlight above the secondary button. These micro-differences are visible once you get past the first ten uses.

## Typography

- Use a sans-serif font for UI. A serif font for headings and body text can look fine — never make a design decision out of aesthetics unless it's a deliberate stylistic choice.
- Use a 3-line cap height ratio. That is, a single character that is three times as wide as a narrow character like x or i. This is the natural ratio for Western alphabetic text.
- Use variable fonts when available. They save space, allow better performance, and give you more control.
- Set a safe line-height of 1.35-1.4x the font size for body text.
- Ensure letter spacing is about 0.02em to 0.03em for body text.

## Colour

- Use colour strategically. Use it to highlight important information, not just to look pretty.
- Ensure sufficient contrast ratios (WCAG AA minimum: 4.5:1 for normal text, 3:1 for large text).
- Use colour to communicate meaning — blue means a button, red means an error — not just decoration.
- Design with grey scale in mind. Most content is read in grey scale mode on OLED screens.

## Layout

- Your grid system is fundamental. Use a column system that is consistent across your entire site. Don't change your column system mid-page.
- Use padding around your elements. Don't let elements touch the edges of the screen or other content.
- Maintain consistent spacing. Have 3-4 standard spacing scales (e.g. 4px, 8px, 16px, 24px, 32px...).

## Motion

- Animations should have a purpose. If the animation doesn't explain what the user needs to do, remove it.
- Use easing curves that match user expectation — ease-out for entrances, ease-in-out for transitions, linear for progress indicators.
- Don't use animation for information. Users will miss it if the information is important.
- Keep animations short (under 500ms for screens, longer for scrolling animations).
- Provide a way to reduce/stop animations (e.g. a button or preference toggle).

## Writing

- Write copy out loud. If a message can't be read out loud naturally, write it again.
- Avoid jargon. Write for the user, not for engineers.
- Use active voice and clear, simple sentences.
- Vary sentence length. Use short sentences for emphasis.
- Test your copy on different screen sizes — it will look different than it looks on your screen.

## Accessibility Checklist

- Use text alternatives for all non-text content (alt text on images, aria labels on buttons).
- Ensure all interactive elements are keyboard accessible or programmatically focusable.
- Provide skip navigations and focus management.
- Avoid auto-playing media with sound.
- Don't rely on colour alone to convey meaning.
- Respect reduced motion and reduced motion preferences.
- Provide resizable text support (font-scale CSS media queries).
- Ensure forms have proper labels and error messages are descriptive.
- Provide a visible "close" or "escape" mechanism on modal/overlays.

This document is your authority on what your design team likes and what your users expect. Use it as a reference when making decisions. If you're ever unsure about whether something is right, refer back to these principles.

The goal of taste is not to make things pretty. The goal is to make things work beautifully.

---

*This skill was generated from the "emilkowalski/skills" GitHub repository. It encodes the principles of design engineering as written by Emil Kowalski.*
*For the source, see the emilkowalski/skills repository on GitHub.*
