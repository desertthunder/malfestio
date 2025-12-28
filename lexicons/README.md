# Lexicon Schemas

This directory contains the Lexicon definitions for the malfestio's public records.

## Evolution Rules

1. **Additive Changes Only**: You can add new optional fields to existing records.
2. **No Renaming**: Do not rename fields.
   If a semantic change is needed, add a new field and deprecate the old one.
3. **No Type Changes**: Once published, a field's type is fixed.
4. **Version by Copying**: If a breaking change is absolutely required, create a new Lexicon with a new major version or a new name (e.g., `app.malfestio.noteV2`).
