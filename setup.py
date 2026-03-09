from setuptools import setup
from setuptools_rust import Binding, RustExtension  # type: ignore

setup(
    rust_extensions=[
        RustExtension(
            "spooky_chess",
            binding=Binding.PyO3,
            debug=False,
            features=["python"],
            rustc_flags=["-Copt-level=3", "-Clto=fat"],
        )
    ],
    data_files=[("", ["spooky_chess.pyi"])],
    zip_safe=False,
)
