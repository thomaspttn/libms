from bisect import bisect_left, bisect_right

import numpy as np
from pyopenms import IonSource, MSExperiment, MzMLFile, PeakPickerHiRes
from scipy import interpolate


class MSData:
    """Class to hold MS data also save a file/sample"""

    def __init__(self, mzML_path: str) -> None:
        """Given path to a .mzML file initialize a PyOpenMS MSExperiment object and extract the pos and neg spectra"""
        self.mzML_path = mzML_path
        exp_temp = MSExperiment()
        try:
            MzMLFile().load(mzML_path, exp_temp)
        except Exception as e:
            raise FileNotFoundError(f"Error loading mzML file: {e}")

        # Get TIC
        tic = exp_temp.calculateTIC()
        tic_rt, tic_inty = tic.get_peaks()
        self.tic_counts = self.process_rt_signal(tic_rt, tic_inty)

        # Centroid entire experiment
        self.exp = MSExperiment()
        PeakPickerHiRes().pickExperiment(exp_temp, self.exp, True)

        pos = [
            spec
            for spec in self.exp
            if spec.getInstrumentSettings().getPolarity() == IonSource.Polarity.POSITIVE
        ]
        neg = [
            spec
            for spec in self.exp
            if spec.getInstrumentSettings().getPolarity() == IonSource.Polarity.NEGATIVE
        ]

        self.pos_spectra = []
        for spec in pos:
            if spec.getMSLevel() == 1:
                mz, intensity = spec.get_peaks()
                rt = spec.getRT()
                self.pos_spectra.append((mz, intensity, rt))

        self.neg_spectra = []
        for spec in neg:
            if spec.getMSLevel() == 1:
                mz, intensity = spec.get_peaks()
                rt = spec.getRT()
                self.neg_spectra.append((mz, intensity, rt))

    def get_spectra(self):
        # Extract positive and negative spectra
        pos_spectra = [
            spec
            for spec in self.exp
            if spec.getInstrumentSettings().getPolarity() == IonSource.Polarity.POSITIVE
        ]
        neg_spectra = [
            spec
            for spec in self.exp
            if spec.getInstrumentSettings().getPolarity() == IonSource.Polarity.NEGATIVE
        ]

        data = []
        for mode, spectra in zip(["pos", "neg"], [pos_spectra, neg_spectra]):
            for spec in spectra:
                mz, inty = spec.get_peaks()
                rt = spec.getRT()
                level = spec.getMSLevel()
                precursor = None
                collision_level = None

                # For MS2, we capture the precursor and collision energy
                if level == 2:
                    if spec.getPrecursors():
                        precursor = spec.getPrecursors()[0].getMZ()
                    else:
                        raise ValueError(
                            f"MS2 spectrum without precursor information in {self.mzML_path}"
                        )
                    if (
                        spec.getPrecursors()
                        and spec.getPrecursors()[0].getActivationMethods()
                    ):
                        collision_level = spec.getPrecursors()[0].getActivationEnergy()

                data.append(
                    {
                        "mz_array": np.array(mz, dtype=np.float32),
                        "inty_array": np.array(inty, dtype=np.float32),
                        "rt": np.array(rt, dtype=np.float32),
                        "mode": mode,
                        "level": level,
                        "precursor": precursor,
                        "collision_level": collision_level,
                    }
                )

        return data

    def get_tic(self, mode):
        """Return TIC for the given mode and mslevel"""
        if mode == "pos":
            spectra = self.pos_spectra
        elif mode == "neg":
            spectra = self.neg_spectra
        else:
            raise ValueError("mode must be 'pos' or 'neg'")

        rt, tic = [], []

        for spec in spectra:
            rt.append(spec[2])
            tic.append(np.sum(spec[1]))

        rt = np.array(rt).astype(np.float32)
        tic = np.array(tic).astype(np.float32)

        _, tic_counts = self.process_rt_signal(rt, tic)

        return tic_counts

    def get_counts_from_target(
        self,
        target_mz: float,
        ppm: float = 4.0,
        mz_offset_ppm: float = 0.0,
        mode: str = "pos",
        interpolate_rt: bool = True,
    ):
        """Return rt and counts for target mz given ppm while filtering based on mode"""
        if mode == "pos":
            spectra = self.pos_spectra
        elif mode == "neg":
            spectra = self.neg_spectra
        else:
            raise ValueError("mode must be 'pos' or 'neg'")

        target_mz = target_mz * (1 + (mz_offset_ppm / 1e6))
        abs_mass_tol = (ppm * target_mz) / 1e6
        start_mz, end_mz = (target_mz - abs_mass_tol, target_mz + abs_mass_tol)
        counts, rt = [], []

        for spec in spectra:
            # Get mz and intensity values
            mz_vals, intensity_vals = spec[0], spec[1]

            if (target_mz <= 51.0) | (target_mz >= 999.0):
                counts.append(0.0)
                rt.append(spec[2])
            else:
                srt_idx, stp_idx = (
                    bisect_left(mz_vals, start_mz),
                    bisect_right(mz_vals, end_mz),
                )
                counts.append(np.max(intensity_vals[srt_idx:stp_idx], initial=0.0))
                rt.append(spec[2])

        rt = np.array(rt).astype(np.float32)
        counts = np.array(counts).astype(np.float32)

        if interpolate_rt:
            rt, counts = self.process_rt_signal(rt, counts)

        return {
            "counts": counts,
            "rt": rt,
        }

    def process_rt_signal(self, rt, intensity, seq_len=2048, max_rt=402.0):
        """Interpolate for even times spacing then pad both sides up to seq_len"""
        # Use up to max_rt
        max_rt = min(max_rt, max(rt))
        rt = rt[rt <= max_rt]
        intensity = intensity[: len(rt)]

        # Interpolate, resample and pad
        f = interpolate.interp1d(rt, intensity, kind="linear", fill_value="extrapolate")
        rt_new = np.arange(min(rt), max(rt), 0.2)  # resample at 0.2s
        intensity_interpolated = f(rt_new)
        pad_left = (seq_len - len(intensity_interpolated)) // 2
        pad_right = seq_len - pad_left - len(intensity_interpolated)
        intensity_padded = np.pad(
            intensity_interpolated, (pad_left, pad_right), mode="constant"
        )
        rt_padded = np.pad(rt_new, (pad_left, pad_right), mode="constant")

        return rt_padded.astype(np.float32), intensity_padded.astype(np.float32)

    def get_metadata(self):
        """Return metadata for the experiment"""
        pass
        # return mzMLMetadata(self.exp).get_metadata()

    # def __len__(self):
    #     return len(self.exp)

    def __getitem__(self, key):
        return self.exp[key]

    # def __repr__(self) -> str:
    #     return f"MSData object with {len(self)} spectra"


if __name__ == "__main__":
    mzML_path = "sample01.mzML"
    ms_data = MSData(mzML_path)
    print(ms_data.get_spectra())
