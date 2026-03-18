#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import re
import sys
from dataclasses import dataclass
from pathlib import Path


WORD_PATTERN = re.compile(r"[A-Za-z]+")
PRONUNCIATION_PATTERN = re.compile(r"\[([^\]]+)\]")
PARAGRAPH_HEADER_PATTERN = re.compile(r"^##\s+(Paragraph\s+\d+)\s*$")
SCRIPT_DIR = Path(__file__).resolve().parent
DEFAULT_OVERRIDES_PATH = SCRIPT_DIR / "pronunciation_overrides.tsv"


@dataclass(frozen=True)
class TranslationRow:
    label: str
    english: str
    concilium: str
    pronunciation: str


@dataclass(frozen=True)
class AudioEntry:
    unit: str
    english: str
    concilium: str
    pronunciation: str
    prompt: str
    phonemes: str
    filename: str


@dataclass(frozen=True)
class PronunciationOverrides:
    by_word: dict[str, str]
    by_pronunciation: dict[str, str]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate Kokoro audio from Concilium markdown tables."
    )
    parser.add_argument(
        "--input",
        default="Sentences.md",
        help="Markdown table to read from. Defaults to Sentences.md.",
    )
    parser.add_argument(
        "--unit",
        choices=("sentences", "paragraphs", "words"),
        default="sentences",
        help="Render sentence rows, paragraph rows, or isolated words. Defaults to sentences.",
    )
    parser.add_argument(
        "--output-dir",
        default=None,
        help="Directory where .wav files will be written. Defaults depend on --unit.",
    )
    parser.add_argument(
        "--voice",
        default="am_onyx",
        help="Kokoro voice name. Must match the selected language code.",
    )
    parser.add_argument(
        "--lang-code",
        default="a",
        help="Kokoro language code. 'a' is American English.",
    )
    parser.add_argument(
        "--device",
        choices=("cpu", "cuda", "auto"),
        default="cpu",
        help="Inference device. Defaults to cpu to avoid surprise GPU issues.",
    )
    parser.add_argument(
        "--speed",
        type=float,
        default=0.9,
        help="Speech speed multiplier.",
    )
    parser.add_argument(
        "--prompt-style",
        choices=("bilingual", "concilium", "english"),
        default="bilingual",
        help="Sentence prompt style. Defaults to bilingual: English first, then Concilium.",
    )
    parser.add_argument(
        "--concilium-only",
        action="store_true",
        help="Skip the English lead-in and render Concilium only.",
    )
    parser.add_argument(
        "--concilium-render",
        choices=("phonemes", "spoken"),
        default="phonemes",
        help="How to render the Concilium side. Defaults to phonemes.",
    )
    parser.add_argument(
        "--overrides",
        default=str(DEFAULT_OVERRIDES_PATH),
        help="TSV file with manual spoken-form overrides.",
    )
    parser.add_argument(
        "--export-overrides-template",
        default=None,
        help="Write a TSV template of unique Concilium words and exit.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=0,
        help="Optional cap for quick test runs. 0 means no limit.",
    )
    return parser.parse_args()


def slugify(value: str, fallback: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return slug or fallback


def normalize_pronunciation(chunk: str) -> str:
    return re.sub(r"\s+", "", chunk.strip().lower())


def load_rows(path: Path) -> list[TranslationRow]:
    text = path.read_text(encoding="utf-8")
    rows = load_table_rows(text)
    if rows:
        return rows

    rows = load_paragraph_rows(text)
    if rows:
        return rows

    raise ValueError(f"no translation rows found in {path}")


def load_table_rows(text: str) -> list[TranslationRow]:
    rows: list[TranslationRow] = []

    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line.startswith("|"):
            continue
        if line.startswith("| ---"):
            continue

        parts = [part.strip() for part in line.strip("|").split("|")]
        if len(parts) != 3:
            continue
        if parts[2] == "Pronunciation":
            continue

        rows.append(
            TranslationRow(
                label=f"Row {len(rows) + 1}",
                english=parts[0],
                concilium=parts[1],
                pronunciation=parts[2],
            )
        )

    return rows


def load_paragraph_rows(text: str) -> list[TranslationRow]:
    rows: list[TranslationRow] = []
    current_label = ""
    current_english = ""
    current_concilium = ""
    current_pronunciation = ""

    def flush_current() -> None:
        nonlocal current_label, current_english, current_concilium, current_pronunciation
        if current_english and current_concilium and current_pronunciation:
            rows.append(
                TranslationRow(
                    label=current_label or f"Paragraph {len(rows) + 1}",
                    english=current_english,
                    concilium=current_concilium,
                    pronunciation=current_pronunciation,
                )
            )
        current_label = ""
        current_english = ""
        current_concilium = ""
        current_pronunciation = ""

    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line:
            continue

        header_match = PARAGRAPH_HEADER_PATTERN.match(line)
        if header_match:
            flush_current()
            current_label = header_match.group(1)
            continue

        if line.startswith("English: "):
            current_english = line.removeprefix("English: ").strip()
            continue

        if line.startswith("Concilium: "):
            current_concilium = line.removeprefix("Concilium: ").strip()
            continue

        if line.startswith("Pronunciation: "):
            current_pronunciation = line.removeprefix("Pronunciation: ").strip()
            continue

    flush_current()
    return rows


def pronounce_token(token: str) -> str:
    token_map = {
        "ah": "ah",
        "eh": "eh",
        "ee": "ee",
        "oh": "oh",
        "oo": "oo",
        "eye": "eye",
        "ow": "ow",
        "kh": "kh",
        "sh": "sh",
        "zh": "zh",
        "ts": "ts",
        "tl": "tl",
        "dr": "dr",
        "kr": "kr",
    }
    return token_map.get(token, token)


def spoken_from_pronunciation(chunk: str) -> str:
    tokens = [token for token in chunk.split("-") if token]
    return "".join(pronounce_token(token) for token in tokens)


def pronunciation_token_to_phoneme(token: str) -> str:
    phoneme_map = {
        "a": "ɑ",
        "ah": "ɑ",
        "e": "e",
        "eh": "e",
        "i": "i",
        "ee": "i",
        "o": "o",
        "oh": "o",
        "u": "u",
        "oo": "u",
        "eye": "aɪ",
        "ow": "aʊ",
        "kh": "x",
        "sh": "ʃ",
        "zh": "ʒ",
        "ts": "ts",
        "tl": "tl",
        "dr": "dɹ",
        "kr": "kɹ",
        "r": "ɹ",
        "g": "ɡ",
        "y": "j",
    }
    return phoneme_map.get(token, token)


def phonemes_from_chunk(chunk: str) -> str:
    tokens = [token for token in chunk.split("-") if token]
    return "".join(pronunciation_token_to_phoneme(token) for token in tokens)


def phonemes_from_pronunciation_text(text: str) -> str:
    segments: list[str] = []

    for chunk, punctuation in re.findall(r"\[([^\]]+)\]([.,!?;:]?)", text):
        phonemes = phonemes_from_chunk(chunk)
        if punctuation:
            phonemes += punctuation
        segments.append(phonemes)

    return " ".join(segments).strip()


def trailing_punctuation(text: str) -> str:
    stripped = text.strip()
    if stripped and stripped[-1] in ".!?":
        return stripped[-1]
    return ""


def parse_word_pronunciations(row: TranslationRow) -> list[tuple[str, str]]:
    words = WORD_PATTERN.findall(row.concilium)
    chunks = PRONUNCIATION_PATTERN.findall(row.pronunciation)

    if chunks and len(words) != len(chunks):
        raise ValueError(
            "word/pronunciation mismatch in row: "
            f"{row.concilium!r} vs {row.pronunciation!r}"
        )

    return list(zip(words, chunks))


def load_overrides(path: Path) -> PronunciationOverrides:
    if not path.exists():
        return PronunciationOverrides(by_word={}, by_pronunciation={})

    by_word: dict[str, str] = {}
    by_pronunciation: dict[str, str] = {}

    with path.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(
            (line for line in handle if line.strip() and not line.lstrip().startswith("#")),
            delimiter="\t",
        )

        for row in reader:
            if row is None:
                continue

            word = (row.get("concilium") or row.get("word") or "").strip()
            pronunciation = (row.get("pronunciation") or "").strip()
            spoken = (row.get("spoken") or row.get("prompt") or "").strip()

            if not spoken:
                continue
            if word:
                by_word[word.lower()] = spoken
            if pronunciation:
                by_pronunciation[normalize_pronunciation(pronunciation)] = spoken

    return PronunciationOverrides(by_word=by_word, by_pronunciation=by_pronunciation)


def resolve_spoken_word(word: str, pronunciation: str, overrides: PronunciationOverrides) -> str:
    by_word = overrides.by_word.get(word.lower())
    if by_word:
        return by_word

    by_pronunciation = overrides.by_pronunciation.get(normalize_pronunciation(pronunciation))
    if by_pronunciation:
        return by_pronunciation

    return spoken_from_pronunciation(pronunciation)


def build_concilium_prompt(row: TranslationRow, overrides: PronunciationOverrides) -> str:
    pairs = parse_word_pronunciations(row)
    if not pairs:
        return row.concilium.strip()

    spoken_words = [
        resolve_spoken_word(word, pronunciation, overrides)
        for word, pronunciation in pairs
    ]
    prompt = " ".join(spoken_words)
    punctuation = trailing_punctuation(row.pronunciation) or trailing_punctuation(row.concilium)
    if punctuation:
        prompt += punctuation
    return prompt


def sentence_prompt(
    row: TranslationRow,
    prompt_style: str,
    overrides: PronunciationOverrides,
) -> str:
    concilium_prompt = build_concilium_prompt(row, overrides)
    english_prompt = row.english.strip()

    if prompt_style == "english":
        return english_prompt
    if prompt_style == "concilium":
        return concilium_prompt
    if prompt_style == "bilingual":
        return f"{english_prompt} {concilium_prompt}".strip()
    raise ValueError(f"unsupported prompt style: {prompt_style}")


def extract_sentence_entries(
    path: Path,
    prompt_style: str,
    overrides: PronunciationOverrides,
) -> list[AudioEntry]:
    entries: list[AudioEntry] = []

    for index, row in enumerate(load_rows(path), start=1):
        filename = f"{index:03d}-{slugify(row.english, f'sentence-{index:03d}')}.wav"
        entries.append(
            AudioEntry(
                unit="sentence",
                english=row.english,
                concilium=row.concilium,
                pronunciation=row.pronunciation,
                prompt=sentence_prompt(row, prompt_style, overrides),
                phonemes=phonemes_from_pronunciation_text(row.pronunciation),
                filename=filename,
            )
        )

    return entries


def extract_paragraph_entries(
    path: Path,
    prompt_style: str,
    overrides: PronunciationOverrides,
) -> list[AudioEntry]:
    entries: list[AudioEntry] = []

    for index, row in enumerate(load_rows(path), start=1):
        filename = f"{index:03d}-{slugify(row.label, f'paragraph-{index:03d}')}.wav"
        entries.append(
            AudioEntry(
                unit="paragraph",
                english=row.english,
                concilium=row.concilium,
                pronunciation=row.pronunciation,
                prompt=sentence_prompt(row, prompt_style, overrides),
                phonemes=phonemes_from_pronunciation_text(row.pronunciation),
                filename=filename,
            )
        )

    return entries


def extract_word_entries(path: Path, overrides: PronunciationOverrides) -> list[AudioEntry]:
    unique_words: dict[str, AudioEntry] = {}

    for row in load_rows(path):
        for word, pronunciation in parse_word_pronunciations(row):
            unique_words.setdefault(
                word,
                AudioEntry(
                    unit="word",
                    english="",
                    concilium=word,
                    pronunciation=f"[{pronunciation}]",
                    prompt=resolve_spoken_word(word, pronunciation, overrides),
                    phonemes=phonemes_from_chunk(pronunciation),
                    filename=f"{slugify(word, 'word')}.wav",
                ),
            )

    return sorted(unique_words.values(), key=lambda entry: entry.concilium.lower())


def extract_entries(
    path: Path,
    unit: str,
    prompt_style: str,
    overrides: PronunciationOverrides,
) -> list[AudioEntry]:
    if unit == "sentences":
        return extract_sentence_entries(path, prompt_style, overrides)
    if unit == "paragraphs":
        return extract_paragraph_entries(path, prompt_style, overrides)
    if unit == "words":
        return extract_word_entries(path, overrides)
    raise ValueError(f"unsupported unit: {unit}")


def export_overrides_template(path: Path, rows: list[TranslationRow]) -> None:
    unique_rows: dict[str, tuple[str, str]] = {}

    for row in rows:
        for word, pronunciation in parse_word_pronunciations(row):
            unique_rows.setdefault(
                word.lower(),
                (word, pronunciation),
            )

    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t")
        writer.writerow(["concilium", "pronunciation", "spoken"])
        for word, pronunciation in sorted(unique_rows.values(), key=lambda item: item[0].lower()):
            writer.writerow([word, f"[{pronunciation}]", spoken_from_pronunciation(pronunciation)])


def import_dependencies() -> tuple[object, object]:
    try:
        from kokoro import KPipeline
        import soundfile as sf
    except ImportError as exc:
        missing = str(exc).split()[-1].strip("'")
        raise SystemExit(
            "Missing dependency: "
            f"{missing}\n"
            "Install Kokoro first. A CPU-only setup is usually the lightest option:\n"
            "  cd gen\n"
            "  python3 -m venv venv\n"
            "  source venv/bin/activate\n"
            "  python -m ensurepip --upgrade\n"
            "  pip install torch --index-url https://download.pytorch.org/whl/cpu\n"
            "  pip install 'kokoro>=0.3.4' soundfile\n"
            "Kokoro also expects espeak-ng to be available on the system."
        ) from exc

    return KPipeline, sf


def collect_audio(generator: object) -> tuple[list[float], int]:
    import numpy as np

    audio_chunks = []
    sample_rate = 24_000

    for result in generator:
        audio = getattr(result, "audio", None)
        if audio is None and getattr(result, "output", None) is not None:
            audio = result.output.audio
        if audio is not None:
            if hasattr(audio, "detach"):
                audio = audio.detach().cpu().numpy()
            else:
                audio = np.asarray(audio)
            audio_chunks.append(audio)

    if not audio_chunks:
        raise RuntimeError("kokoro returned no audio")

    if len(audio_chunks) == 1:
        return audio_chunks[0], sample_rate

    import numpy as np

    return np.concatenate(audio_chunks), sample_rate


def render_audio(
    entry: AudioEntry,
    pipeline: object,
    voice: str,
    speed: float,
    prompt_style: str,
    concilium_render: str,
) -> tuple[list[float], int]:
    import numpy as np

    def render_text(text: str) -> tuple[list[float], int]:
        return collect_audio(
            pipeline(
                text,
                voice=voice,
                speed=speed,
                split_pattern=r"$",
            )
        )

    def render_phonemes(phonemes: str) -> tuple[list[float], int]:
        return collect_audio(
            pipeline.generate_from_tokens(
                phonemes,
                voice=voice,
                speed=speed,
            )
        )

    if entry.unit == "word":
        if concilium_render == "phonemes":
            return render_phonemes(entry.phonemes)
        return render_text(entry.prompt)

    if prompt_style == "english":
        return render_text(entry.english)

    if prompt_style == "concilium":
        if concilium_render == "phonemes":
            return render_phonemes(entry.phonemes)
        return render_text(entry.prompt)

    if prompt_style == "bilingual":
        if concilium_render == "spoken":
            return render_text(entry.prompt)

        english_audio, sample_rate = render_text(entry.english)
        concilium_audio, _ = render_phonemes(entry.phonemes)
        pause = np.zeros(int(sample_rate * 0.18), dtype=english_audio.dtype)
        return np.concatenate([english_audio, pause, concilium_audio]), sample_rate

    raise ValueError(f"unsupported render combination: {prompt_style} / {concilium_render}")


def write_manifest(path: Path, entries: list[AudioEntry]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t")
        writer.writerow(["unit", "english", "concilium", "pronunciation", "prompt", "phonemes", "file"])
        for entry in entries:
            writer.writerow(
                [
                    entry.unit,
                    entry.english,
                    entry.concilium,
                    entry.pronunciation,
                    entry.prompt,
                    entry.phonemes,
                    entry.filename,
                ]
            )


def default_output_dir(unit: str) -> Path:
    if unit == "sentences":
        return Path("audio/kokoro_sentences")
    if unit == "paragraphs":
        return Path("audio/kokoro_paragraphs")
    if unit == "words":
        return Path("audio/kokoro_words")
    raise ValueError(f"unsupported unit: {unit}")


def main() -> int:
    args = parse_args()
    prompt_style = "concilium" if args.concilium_only else args.prompt_style
    input_path = Path(args.input)
    output_dir = Path(args.output_dir) if args.output_dir else default_output_dir(args.unit)
    overrides_path = Path(args.overrides)
    rows = load_rows(input_path)

    if args.export_overrides_template:
        template_path = Path(args.export_overrides_template)
        export_overrides_template(template_path, rows)
        print(f"Wrote override template: {template_path}")
        return 0

    output_dir.mkdir(parents=True, exist_ok=True)
    overrides = load_overrides(overrides_path)
    entries = extract_entries(input_path, args.unit, prompt_style, overrides)
    if args.limit > 0:
        entries = entries[: args.limit]

    KPipeline, sf = import_dependencies()
    device = None if args.device == "auto" else args.device
    pipeline = KPipeline(
        lang_code=args.lang_code,
        repo_id="hexgrad/Kokoro-82M",
        device=device,
    )

    print(f"Using overrides: {overrides_path}")
    print(f"Rendering {len(entries)} {args.unit} from {input_path} to {output_dir}...")

    manifest_path = output_dir / "manifest.tsv"
    write_manifest(manifest_path, entries)

    for index, entry in enumerate(entries, start=1):
        audio, sample_rate = render_audio(
            entry,
            pipeline,
            args.voice,
            args.speed,
            prompt_style,
            args.concilium_render,
        )
        target = output_dir / entry.filename
        sf.write(target, audio, sample_rate)
        print(f"[{index}/{len(entries)}] wrote {target}")

    print(f"Manifest: {manifest_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
