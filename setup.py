from setuptools import setup
from setuptools_rust import Binding, RustExtension  # type: ignore

setup(
    rust_extensions=[
        RustExtension(
            "rust_chess",
            binding=Binding.PyO3,
            debug=False,
            features=["python"],
        )
    ],
    data_files=[("", ["rust_chess.pyi"])],
    zip_safe=False,
)
