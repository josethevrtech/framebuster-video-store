import importlib.util
import io
from pathlib import Path
import tempfile
import unittest
from zipfile import ZipFile

spec = importlib.util.spec_from_file_location('importer', Path(__file__).with_name('import-retro-library.py'))
importer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(importer)


class LibraryImportTest(unittest.TestCase):
    def test_release_filter_and_multidisc_integrity(self):
        root = Path(tempfile.mkdtemp(prefix='rom-import-test-', dir=Path(__file__).resolve().parents[2]))
        metadata = root / 'metadata'
        metadata.mkdir()
        (metadata / 'test.dat').write_text('game ( comment "Old (USA)" releaseyear "1999" )\n'
                                         'game ( comment "New (USA)" releaseyear "2000" )')
        with ZipFile(root / 'test.zip', 'w') as archive:
            for title in ['Old', 'New', 'Unknown']:
                contents = io.BytesIO()
                with ZipFile(contents, 'w') as nested:
                    nested.writestr(title + ' (USA).z64', title.encode())
                archive.writestr(title + ' (USA).zip', contents.getvalue())
        for name in ['Sony - PS1 (A-L).zip', 'Sony - PS1 (L-Z).zip', 'Sony - PS1 (Update 1).zip']:
            with ZipFile(root / name, 'w') as archive:
                archive.writestr('gamelist.xml', '<gameList><game><path>./Old.m3u</path><name>Old</name>'
                                 '<releasedate>19991231T000000</releasedate></game>'
                                 '<game><path>./Missing.m3u</path><name>Missing</name>'
                                 '<releasedate>19980101T000000</releasedate></game></gameList>')
                archive.writestr('Old.m3u', 'Disc 1.chd\nDisc 2.chd\n')
                archive.writestr('Disc 1.chd', 'one')
                archive.writestr('Disc 2.chd', 'two')
                archive.writestr('Missing.m3u', 'Absent.chd\n')
        original = importer.SYSTEMS
        importer.SYSTEMS = [('test.zip', 'n64', 'test')]
        try:
            result = importer.plan(root, metadata)
        finally:
            importer.SYSTEMS = original
        self.assertEqual({i['destination'] for i in result['files']},
                         {'n64/Old (USA).z64', 'psx/Old.m3u', 'psx/Disc 1.chd', 'psx/Disc 2.chd'})
        self.assertEqual(len(result['files']), 4)
        self.assertTrue(any(i[2] == 'missing disc' for i in result['skipped']))


if __name__ == '__main__':
    unittest.main()
