from pathlib import Path
from zipfile import ZipFile
import importlib.util
import tempfile
import unittest


spec=importlib.util.spec_from_file_location('artwork_import',Path(__file__).with_name('import-private-artwork.py'))
module=importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class ArtworkImport(unittest.TestCase):
    def setUp(self):
        self.base=Path(tempfile.mkdtemp(prefix='framebuster-artwork-test-'))
        self.root=self.base/'library'
        self.root.mkdir()
        self.archive=self.base/'art.zip'

    def zip(self,items):
        with ZipFile(self.archive,'w') as archive:
            for name,value in items: archive.writestr(name,value)

    def test_existing_art_is_preserved_and_matching_retries_are_idempotent(self):
        self.zip([('artwork/game.rgba',b'original artwork')])
        self.assertEqual(module.import_artwork(self.archive,self.root),1)
        self.assertEqual(module.import_artwork(self.archive,self.root),0)
        self.assertEqual((self.root/'artwork/game.rgba').read_bytes(),b'original artwork')

    def test_conflicting_art_fails_before_writing_new_files(self):
        (self.root/'artwork').mkdir()
        (self.root/'artwork/game.rgba').write_bytes(b'keep this')
        self.zip([('artwork/new.rgba',b'new'),('artwork/game.rgba',b'conflict')])
        with self.assertRaises(ValueError): module.import_artwork(self.archive,self.root)
        self.assertEqual((self.root/'artwork/game.rgba').read_bytes(),b'keep this')
        self.assertFalse((self.root/'artwork/new.rgba').exists())

    def test_paths_cannot_escape_the_private_library(self):
        self.zip([('artwork/../../escaped.txt',b'bad')])
        with self.assertRaises(ValueError): module.import_artwork(self.archive,self.root)
        self.assertFalse((self.base/'escaped.txt').exists())

if __name__=='__main__': unittest.main()
