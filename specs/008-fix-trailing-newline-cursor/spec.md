# Feature Specification: Fix Trailing Newline Cursor Position

**Feature Branch**: `008-fix-trailing-newline-cursor`

**Created**: 2026-07-03

**Status**: Draft

**Input**: User description: "Es gibt einen Fehler am Ende der Datei. Endet eine Datei mit einem Newline, so bleibt der Cursor am Ende der Zeile davor. Erst wenn man ein Zeichen eingibt, wird dieses an die korrekte Stelle geschrieben und der Cursor ist dahinter. Nur, wenn der Cursor hinter dem letzten Newline steht, ist die Position falsch"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Cursor korrekt hinter abschließendem Newline platzieren (Priority: P1)

Ein Nutzer öffnet eine Datei, deren Inhalt mit einem Newline-Zeichen endet (die letzte Zeile ist danach leer). Der Cursor wird ans Ende des Dokuments bewegt, also genau hinter das letzte Newline. In diesem Zustand MUSS der Cursor visuell am Anfang der leeren Abschlusszeile stehen und dort platziert sein, nicht am Ende der vorangehenden, nicht-leeren Zeile. Die dargestellte Cursorposition stimmt mit der logischen Einfügeposition überein.

**Why this priority**: Dieser Fehlerfall betrifft genau die beschriebene Kernsituation (Cursor hinter dem letzten Newline). Ohne Korrektur gibt der Editor eine falsche Cursorposition vor, was den Nutzer beim Bearbeiten des Dateiendes irritiert und zu versehentlich fehlerplatzierten Eingaben führt. Diese Reise liefert für sich allein den nutzbaren Mehrwert eines korrekt angezeigten Cursors am Dateiende.

**Independent Test**: Eine Datei öffnen, deren Inhalt auf ein Newline endet (z. B. "abc\n"), den Cursor ans Dokumentende bewegen und prüfen, dass der Cursor am Anfang der leeren Abschlusszeile steht. Erwartete Wirkung: Die Cursoranzeige entspricht der logischen Position hinter dem letzten Newline.

**Acceptance Scenarios**:

1. **Given** ein offenes Dokument mit Inhalt "abc\n" (endet mit abschließendem Newline), **When** der Nutzer den Cursor bis ans Ende des Dokuments bewegt (hinter das letzte Newline), **Then** steht der Cursor visuell am Anfang der leeren Abschlusszeile und nicht am Ende der Zeile "abc".
2. **Given** der Cursor steht exakt hinter dem letzten Newline eines Dokuments, **When** der Nutzer ein Zeichen eingibt (z. B. "x"), **Then** wird "x" direkt an der korrekten Position, also am Anfang der leeren Abschlusszeile, eingefügt und der Cursor steht dahinter – ohne dass das Zeichen zuvor an einer falschen Stelle erscheint.
3. **Given** ein Dokument endet mit abschließendem Newline und der Cursor wurde ans Ende bewegt, **When** der Nutzer den Cursor nach links bewegt, **Then** springt der Cursor korrekt an das Ende der vorherigen Zeile (vor das abschließende Newline) und die-navigation verhält sich widerspruchsfrei.

---

### User Story 2 - Eingabe am Ende der Zeile vor dem abschließenden Newline (Priority: P2)

Ein Nutzer bearbeitet eine Datei, die mit einem Newline endet, und platziert den Cursor am Ende der letzten nicht-leeren Zeile (also direkt vor dem abschließenden Newline, nicht dahinter). In diesem Zustand MUSS eine Eingabe korrekt am Zeilenende eingefügt werden und der Cursor dahinter stehen. Dieser Fall funktioniert bereits fehlerfrei und MUSS durch die Fehlerbehebung nicht regressiv verändert werden.

**Why this priority**: Die Nutzerbeschreibung stellt klar, dass die fehlerhafte Position nur auftritt, wenn der Cursor *hinter* dem letzten Newline steht. Dieser Story sichert das weiterhin korrekte Verhalten am Zeilenende und schützt vor Regressionen, die durch eine zu breit angelegte Korrektur entstehen könnten.

**Independent Test**: Eine Datei "abc\n" öffnen, den Cursor an das Ende der Zeile "abc" (vor das Newline) setzen, ein Zeichen eingeben und prüfen, dass es dort eingefügt wird. Erwartete Wirkung: Zeichen steht am Zeilenende, Cursor dahinter, abschließendes Newline bleibt erhalten.

**Acceptance Scenarios**:

1. **Given** ein Dokument "abc\n", Cursor am Ende der Zeile "abc" (vor dem Newline), **When** der Nutzer "x" eingibt, **Then** wird der Inhalt zu "abcx\n" und der Cursor steht hinter dem "x".
2. **Given** ein Dokument "abc\n", Cursor am Ende der Zeile "abc", **When** der Nutzer den Cursor nach rechts bewegt, **Then** springt der Cursor an den Anfang der leeren Abschlusszeile (hinter das Newline) und wird dort korrekt angezeigt (gemäß User Story 1).

---

### Edge Cases

- Was passiert bei einer Datei, die mit mehreren aufeinanderfolgenden Newlines endet (z. B. "abc\n\n\n")? Der Cursor hinter dem letzten Newline MUSS am Anfang der letzten, leeren Zeile stehen; Cursor hinter dem vorletzten Newline MUSS am Anfang der vorletzten leeren Zeile stehen – konsistent über alle leeren Abschlusszeilen.
- Wie verhält sich der Cursor, wenn das Dokument nur aus einem einzelnen Newline ("\n") besteht? Der Cursor hinter diesem Newline MUSS am Anfang der leeren zweiten Zeile stehen; der Cursor davor MUSS am Anfang der ersten (leeren) Zeile stehen.
- Wie verhält sich der Cursor bei einem leeren Dokument ("") ohne jedes Newline? Hier darf kein falsches Verhalten am Dateiende entstehen; der Cursor steht schlicht am Anfang des leeren Dokuments.
- Was passiert bei einer Datei, die nicht mit einem Newline endet (z. B. "abc")? Der Cursor am Ende des Dokuments MUSS hinter dem "c" stehen; dieser Fall MUSS durch die Korrektur nicht verändert werden (keine Regression).
- Was passiert beim Speichern, sodass kein versehentlicher Datenverlust oder versehentliches Überschreiben des abschließenden Newlines entsteht? **Klärung (2026-07-03)**: Dieser Aspekt ist explizit out-of-scope – die Persistenzlogik bleibt unverändert, der Speichern-Pfad erfordert keine neuen Tests. Die bestehenden Speichern-Tests bleiben als Regressionsschutz erhalten.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Der Editor MUSS den Cursor, wenn er hinter dem letzten Zeichen eines Dokuments steht, das mit einem Newline endet, visuell am Anfang der darauffolgenden, leeren Abschlusszeile anzeigen.
- **FR-002**: Der Editor MUSS die logische Cursorposition (Einfügeposition) konsistent mit der dargestellten Cursorposition halten, sodass eine Eingabe an der sichtbaren Stelle und nicht an einer davon abweichenden Position erfolgt.
- **FR-003**: Der Editor MUSS eine Eingabe, wenn der Cursor hinter dem letzten Newline steht, an Anfang der leeren Abschlusszeile einfügen und den Cursor unmittelbar hinter das eingefügte Zeichen setzen – ohne dass das Zeichen zuvor an einer falschen Position erscheint.
- **FR-004**: Der Editor MUSS die Pfeilnavigation am Ende einer Datei mit abschließendem Newline widerspruchsfrei unterstützen: ein Schritt nach rechts über das Newline bewegt den Cursor an den Anfang der leeren Abschlusszeile, ein Schritt zurück an das Ende der vorherigen Zeile.
- **FR-005**: Der Editor MUSS die Korrektur ausschließlich auf den beschriebenen Fehlerfall (Cursor hinter dem letzten Newline) anwenden und das bereits korrekte Verhalten am Ende der letzten nicht-leeren Zeile unverändert lassen.
- **FR-006**: Der Editor MUSS auch bei mehrfachem, abschließendem Newline sowie bei einem Dokument, das nur aus Newlines besteht, eine konsistente Cursorposition zeigen (jeweils am Anfang der nachfolgenden leeren Zeile).
- **FR-007**: Der Editor MUSS vermeiden, dass durch die Cursorpositionskorrektur Datenverlust oder ein versehentliches Überschreiben des abschließenden Newlines beim Speichern entsteht (sichere Persistenz des Dateiinhalts).
- **FR-008**: Automatisierte Tests MUSS den beschriebenen Fehlerfall reproduzieren (Cursor hinter abschließendem Newline), die Korrektur der sichtbaren Cursorposition prüfen sowie die korrekte Einfügeposition bei Eingabe verifizieren; Edge Cases (mehrfaches Newline, einzelnes Newline, nicht mit Newline endend, leeres Dokument) MUSS abgedeckt sein.

### Key Entities

- **Cursor**: Die logische Einfügeposition im Dokument, ausgedrückt als Versatz im Unicode-Codepoint-Stream. Relevant sind hier die Relation zwischen Cursorposition, Zeilenumbrüchen (Newline) und der sichtbaren, zweidimensionalen Cursoranzeige in Zeile/Spalte.
- **Dokument**: Der bearbeitete Textinhalt einer einzelnen Datei, bestehend aus Zeilen, die durch Newline-Zeichen getrennt sind. Relevant ist, ob das Dokument mit einem abschließenden Newline endet und wie viele leere Abschlusszeilen dadurch entstehen.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Nutzer sehen den Cursor an exakt der Position, an der die nächste Eingabe eingefügt wird, in 100 % der Fälle, in denen der Cursor hinter einem abschließenden oder inneren Newline am Dateiende steht.
- **SC-002**: Eine Eingabe nach Bewegen des Cursors ans Dateiende (hinter das abschließende Newline) erscheint direkt am sichtbaren Cursor und verschiebt den Cursor korrekt dahinter, in jedem Testfall ohne sichtbaren "Sprung" der Eingabe an eine andere Stelle.
- **SC-003**: Die Pfeilnavigation am Dateiende verhält sich widerspruchsfrei; der Nutzer kann beliebig zwischen dem Ende der letzten nicht-leeren Zeile und dem Anfang der leeren Abschlusszeile wechseln, ohne dass Cursoranzeige und Einfügeposition auseinanderlaufen.
- **SC-004**: Bei Dateien, die nicht mit einem Newline enden, sowie am Ende der letzten nicht-leeren Zeile bleibt das bestehende, korrekte Verhalten unverändert erhalten (keine Regression, verifiziert durch bestehende Tests).
- **SC-005**: Automatisierte Tests decken den primären Fehlerfluss (Cursor hinter abschließendem Newline) und die definierten Edge Cases (mehrfaches Newline, einzelnes Newline, nicht mit Newline endend, leeres Dokument) ab.

## Clarifications

### Session 2026-07-03

- Q: Benötigt der Speichern-/Persistenz-Pfad neue automatisierte Tests für das abschließende Newline? → A: Nein, explizit out-of-scope: Persistenz bleibt unverändert, keine neuen Speichern-Pfad-Tests erforderlich.

## Assumptions

- Das Dokumentmodell repräsentiert Newlines als explizite Zeichen im Unicode-Codepoint-Stream (wie bereits in frürieren Spezifikationen festgelegt, z. B. UTF-8-Codepoint-basierte Cursorpositionen), und eine leere Abschlusszeile entsteht durch ein abschließendes Newline.
- Die Korrektur betrifft nur die Anzeige-/Positionslogik am Dateiende; die Speicherung des Dokumentinhalts bleibt unverändert (ein abschließendes Newline wird weiterhin als solches gespeichert, ohne automatisches Hinzufügen oder Entfernen). Der Speichern-/Persistenz-Pfad ist explizit out-of-scope für diese Änderung und erfordert keine neuen automatisierten Tests; bestehende Speichern-Tests bleiben als Regressionsschutz erhalten.
- Der Begriff "Cursor hinter dem letzten Newline" bezeichnet die logische Position unmittelbar nach dem abschließenden Newline-Zeichen, also am Anfang der darauffolgenden, leeren Abschlusszeile.
- Die Pfeilnavigation am Zeilenende entspricht dem bereits implementierten Standardverhalten (Cursor nach rechts über ein Newline springt an den Anfang der nächsten Zeile); diese Spezifikation setzt voraus, dass dies auch am Dateiende konsistent gilt.
- Es gibt keine Mehrfach-Cursor- oder Block-Selektionsmodi, die vom Verhalten am Dateiende gesondert betrachtet werden müssen (im Rahmen dieser Fehlereingrenzung nicht relevant).
