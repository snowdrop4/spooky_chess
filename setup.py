from setuptools import setup
from setuptools_rust import Binding, RustExtension  # type: ignore

setup(
    rust_extensions=[
        RustExtension(
            "spooky_chess",
            binding=Binding.PyO3,
            debug=False,
            features=["python"],
        )
    ],
    data_files=[("", ["spooky_chess.pyi"])],
    zip_safe=False,
)
