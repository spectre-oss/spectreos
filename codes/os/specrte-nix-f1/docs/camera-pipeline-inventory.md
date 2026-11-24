# Original Xiaomi camera pipeline inventory

Recovered on 2026-10-20 from the owner's archived MIUI ROM. These are static definitions, not proof that every branch is active on the Poco F1. See [pipeline analysis](camera-pipeline.md) for interpretation and provenance.

## Compiled Xiaomi ZSL use case: all 36 pipelines

The CHI binary SHA-256 is `25643fc83f3a300e5901edb83cd20644bc360b23a87d5a647f7b9b6c6e5d2ad6`. ELF virtual addresses below exclude Ghidra's 0x10000 image base. Node IDs distinguish duplicate instances; plugin names identify custom nodes. Node order here is storage order, not processing order.

| Index | Advanced type | Name | ELF VA | Nodes and custom plugins |
| --- | --- | --- | --- | --- |
| 0 | 28 | `MiuiZSLSnapshotJpeg` | `0x9b240` | ChiNode0 (com.arcsoft.node.skinbeautifier); ChiNode1 (com.xiaomi.node.mibokeh); ChiNode2 (com.qti.node.gpu); ChiNode3 (com.qti.node.watermark); JPEGAggregator0; JPEG0; IPE0; BPS0 |
| 1 | 0 | `ZSLSnapshotJpeg` | `0x9b27c` | JPEGAggregator0; JPEG0; JPEG1; IPE2; BPS0 |
| 2 | 32 | `ZSLSnapshotJpegPure` | `0x9b2b8` | ChiNode0 (com.arcsoft.node.skinbeautifier); ChiNode3 (com.qti.node.gpu); JPEGAggregator0; JPEG0; JPEG1; IPE2; BPS0 |
| 3 | 1 | `ZSLSnapshotYUV` | `0x9b2f4` | IPE0; BPS0 |
| 4 | 35 | `BinningZSLSnapshotYUV` | `0x9b330` | IPE0; BPS0 |
| 5 | 12 | `ZSLSnapshotYUVAux` | `0x9b36c` | IPE0; BPS0 |
| 6 | 23 | `BackCameraJpegEncode` | `0x9b3a8` | ChiNode0 (com.arcsoft.node.skinbeautifier); ChiNode1 (com.qti.node.watermark); JPEGAggregator0; JPEG0 |
| 7 | 2 | `InternalZSLYuv2Jpeg` | `0x9b3e4` | ChiNode0 (com.arcsoft.node.skinbeautifier); ChiNode1 (com.xiaomi.node.mibokeh); ChiNode2 (com.qti.node.gpu); ChiNode3 (com.qti.node.watermark); JPEGAggregator0; JPEG0; IPE0 |
| 8 | 24 | `BinningZSLYuv2Jpeg` | `0x9b420` | ChiNode1 (com.xiaomi.node.mibokeh); ChiNode2 (com.qti.node.gpu); ChiNode3 (com.qti.node.watermark); JPEGAggregator0; JPEG0; IPE0 |
| 9 | 3 | `InternalZSLYuv2JpegMFNR` | `0x9b45c` | ChiNode0 (com.arcsoft.node.skinbeautifier); ChiNode1 (com.qti.node.watermark); ChiNode2 (com.qti.node.gpu); ChiNode3 (com.xiaomi.node.mibokeh); JPEGAggregator0; JPEG0; JPEG1; IPE6 |
| 10 | 13 | `InternalZSLYuv2JpegMFNRAux` | `0x9b498` | ChiNode0 (com.qti.node.watermark); JPEGAggregator0; JPEG0; JPEG1; IPE6 |
| 11 | 22 | `AdvancedAsdMeta` | `0x9b4d4` | ChiNode0 (com.arcsoft.node.hdrchecker.so); ChiNode1 (com.mi.node.aiasd); ChiNode2 (com.mi.node.aiadd); ChiNode3 (com.qti.node.xiaomigenderage) |
| 12 | 29 | `SimpleAdvancedAsdMeta` | `0x9b510` | ChiNode0 (com.arcsoft.node.hdrchecker.so); ChiNode1 (com.mi.node.aiasd); ChiNode2 (com.qti.node.xiaomigenderage) |
| 13 | 18 | `ZSLYuv2Yuv` | `0x9b54c` | IPE0 |
| 14 | 19 | `ZSLSnapshotJpegGPU` | `0x9b588` | ChiNode0 (com.qti.node.gpu); JPEGAggregator0; JPEG0; JPEG1; IPE0; BPS0 |
| 15 | 31 | `ZSLSnapshotJpegBokeh` | `0x9b5c4` | ChiNode0 (com.arcsoft.node.skinbeautifier); ChiNode1 (com.xiaomi.node.mibokeh); ChiNode2 (com.qti.node.gpu); ChiNode3 (com.qti.node.watermark); JPEGAggregator0; JPEG0; IPE0; BPS0 |
| 16 | 8 | `MfnrScale` | `0x9b600` | IPE5 |
| 17 | 15 | `MfnrScaleAux` | `0x9b63c` | IPE5 |
| 18 | 26 | `BinningMerge3YuvCustomTo1Yuv` | `0x9b678` | ChiNode0 (com.arcsoft.node.hdr.so); ChiNode1 (com.arcsoft.node.skinbeautifier) |
| 19 | 7 | `MfnrPrefilter` | `0x9b6b4` | IPE1; BPS1 |
| 20 | 14 | `MfnrPrefilterAux` | `0x9b6f0` | IPE1; BPS1 |
| 21 | 4 | `Merge3YuvCustomTo1Yuv` | `0x9b72c` | ChiNode0 (com.arcsoft.node.hdr.so) |
| 22 | 11 | `SWMFMergeYuv` | `0x9b768` | ChiNode0 (com.qti.node.memcpy) |
| 23 | 30 | `SWMFClearShotYuv` | `0x9b7a4` | ChiNode0 (com.qti.node.clearshot); ChiNode1 (com.arcsoft.node.skinbeautifier) |
| 24 | 27 | `MiuiZSLPreviewRaw` | `0x9b7e0` | Sensor0 (com.qti.stats.pdlibwrapper); StatsProcessing0 (com.qti.stats.aec, com.qti.stats.awbwrapper, com.qti.stats.afd); ChiNode0 (com.arcsoft.node.skinbeautifier); ChiNode1 (com.xiaomi.node.mibokeh); AutoFocus0 (com.qti.stats.af, com.qti.stats.pdlibwrapper); IFE0 (com.qti.hvx.addconstant); IPE0; FDHw0; FDManager0; StatsParse0 |
| 25 | 5 | `ZSLPreviewRaw` | `0x9b81c` | Sensor0 (com.qti.stats.pdlibwrapper); StatsProcessing0 (com.qti.stats.aec, com.qti.stats.awbwrapper, com.qti.stats.afd); AutoFocus0 (com.qti.stats.af, com.qti.stats.pdlibwrapper); IFE0 (com.qti.hvx.addconstant); IPE0; FDHw0; FDManager0; StatsParse0 |
| 26 | 6 | `ZSLPreviewRawYUV` | `0x9b858` | Sensor0 (com.qti.stats.pdlibwrapper); StatsProcessing0 (com.qti.stats.aec, com.qti.stats.awb, com.qti.stats.afd); AutoFocus0 (com.qti.stats.af, com.qti.stats.pdlibwrapper); IFE0; IPE0; FDHw0; LRME0; RANSAC0; FDManager0; StatsParse0 |
| 27 | 34 | `MiuiPreviewRawYUV` | `0x9b894` | Sensor0 (com.qti.stats.pdlibwrapper); StatsProcessing0 (com.qti.stats.aec, com.qti.stats.awb, com.qti.stats.afd); ChiNode0 (com.arcsoft.node.skinbeautifier); ChiNode1 (com.qti.node.gpu); AutoFocus0 (com.qti.stats.af, com.qti.stats.pdlibwrapper); IFE0; IPE0; IPE1; FDHw0; LRME0; RANSAC0; FDManager0; StatsParse0 |
| 28 | 10 | `MfnrPostFilter` | `0x9b8d0` | ChiNode0 (com.qti.node.swregistration); IPE3; IPE4; BPS3; BPS4 |
| 29 | 25 | `BinningMfnrPostFilter` | `0x9b90c` | ChiNode0 (com.qti.node.swregistration); ChiNode1 (com.arcsoft.node.skinbeautifier); IPE3; IPE4; BPS3; BPS4 |
| 30 | 17 | `MfnrPostFilterAux` | `0x9b948` | ChiNode0 (com.qti.node.swregistration); IPE3; IPE4; BPS3; BPS4 |
| 31 | 33 | `SuperNightYuv` | `0x9b984` | ChiNode0 (com.arcsoft.node.superlowlight.so) |
| 32 | 9 | `MfnrBlend` | `0x9b9c0` | ChiNode0 (com.qti.node.swregistration); IPE2; BPS2 |
| 33 | 16 | `MfnrBlendAux` | `0x9b9fc` | ChiNode0 (com.qti.node.swregistration); IPE2; BPS2 |
| 34 | 20 | `SWMFHHTYuv` | `0x9ba38` | ChiNode0 (com.qti.node.photosolid) |
| 35 | 21 | `SWMFSRYuv` | `0x9ba74` | ChiNode0 (com.qti.node.sr) |

## Titan container: all 45 use cases / 115 pipelines

Container SHA-256: `f30ff895b640dd591fb3bd87270f1b72b1c8df8b5e4a7c06a8917bf20375b7d5`. Compiled Xiaomi definitions can replace these generic definitions, including their custom processing plugins.

| Use case | Pipeline names | Decoder warnings |
| --- | --- | --- |
| UsecaseTorch | TorchPipeline | None |
| UsecaseRaw8 | Raw8 | None |
| UsecaseRaw | Raw | None |
| UsecasePreview | Preview | None |
| UsecaseSecurePreview | SecurePreview | None |
| UsecaseVideo | PreviewVideo | None |
| UsecaseVideoHFR | PreviewVideoHFR | None |
| UsecaseVideoHFRNo3A | PreviewVideoHFRNo3A | None |
| UsecaseVideoHFRNo3ANoPreview | PreviewVideoHFRNo3ANoPreview | None |
| UsecaseSnapshot | Snapshot | None |
| UsecaseVideoWithRawCapture | VideoWithRawCapture | None |
| UsecaseReprocessTwoStreams | RealtimeRdi; ReprocessYuv2Streams | None |
| UsecaseReprocessTwoStreamsForJPEG | RealtimeRdiJpeg; ReprocessJpeg | None |
| UsecaseReprocessThreeStreamsNoPreview | ThreeStreamsRealtimeRdi; ReprocessYuv3Streams | None |
| UsecaseReprocessThreeStreamsPreview | RealtimeRdiAndPreview; ReprocessRawAndPreview | None |
| UsecaseJPEGEncodeSnapshot | JPEGEncodeSnapshotOnly | None |
| UsecaseJPEGEncodePreviewSnapshot | JPEGEncodePreview | None |
| UsecaseJPEGEncodeLiveSnapshot | JPEGEncodeLiveSnapshotPreview | JPEGEncodeLiveSnapshotPreview: Original definition repeats node IDs/instances; all rows retained, Endpoint absent or inconsistent in node list: FDHw/0, Endpoint absent or inconsistent in node list: FDManager/0 |
| UsecasePreviewRaw | PreviewRaw | None |
| UsecaseYUV2YUVReprocessTwoStreamsNoPreview | RealtimeYUV2YUV2Streams; ReprocessYuvToYuv2Streams | None |
| UsecaseYUV2YUVReprocessTwoStreamsNoPreviewForBidirectional | RealtimeYUV2YUVReprocess; ReprocessYuv2YuvNoPreview | None |
| UsecaseYUV2YUVReprocessThreeStreamsNoPreviewMoreDownscale | RealtimeYUV2YUVReprocess3StreamsMoreDownscale; ReprocessYuvToYuv3StreamsMoreDownscale | None |
| UsecaseYUV2YUVReprocessThreeStreamsNoPreview | RealtimeYUV2YUVReprocess3Streams; ReprocessYuvToYuv3Streams | None |
| UsecaseYUV2YUVReprocessThreeStreamsWithPreviewForBidirectional | RealtimeAndPreview; ReprocessYUVAndPreview | None |
| UsecaseYUV2JpegReprocessTwoStreamsNoPreview | RealtimeYUV2Jpeg; ReprocessJpeg2Streams | None |
| UsecaseYUV2JpegReprocessThreeStreamsNoPreview | RealtimeYUVToJpeg3Streams; ReprocessJpeg3Streams1; ReprocessJpeg3Streams2 | None |
| UsecaseZSL | ZSLPreviewRaw; ZSLSnapshotJpeg; ZSLSnapshotYUV; ZSLSnapshotYUVAux; InternalZSLYuv2Jpeg; InternalZSLYuv2JpegMFNR; InternalZSLYuv2JpegMFNRAux; Merge3YuvCustomTo1Yuv; SWMFMergeYuv; MfnrPrefilter; MfnrPrefilterAux; MfnrBlend; MfnrBlendAux; MfnrScale; MfnrScaleAux; MfnrPostFilter; MfnrPostFilterAux; ZSLYuv2Yuv; ZSLSnapshotJpegGPU | None |
| UsecaseZSLTuning | ZSLPreviewRawTuning; ZSLSnapshotJpegTuning; ZSLSnapshotYUVTuning | None |
| UsecaseQuadCFA | QuadCFAPreview; QuadCFARealtimeRaw; QuadCFASnapshotJPEG; QuadCFASnapshotYUV | None |
| UsecaseVideoEIS2PreviewEIS2 | VideoEIS2PreviewEIS2 | VideoEIS2PreviewEIS2: Original definition repeats node IDs/instances; all rows retained, Endpoint absent or inconsistent in node list: FDHw/0, Endpoint absent or inconsistent in node list: FDManager/0 |
| UsecaseVideoEIS3PreviewEIS2 | VideoEIS3PreviewEIS2 | VideoEIS3PreviewEIS2: Endpoint absent or inconsistent in node list: FDHw/0, Endpoint absent or inconsistent in node list: FDManager/0 |
| UsecaseMFNR | Prefilter; Blend; Scale; PostFilter | None |
| UsecaseJPEGEncodePreviewSnapshotGPU | JPEGEncodeSnapshotGPU | None |
| UsecaseJPEGEncodeLiveSnapshotGPU | JPEGEncodeLiveSnapshotGPU | None |
| UsecaseRTB | RealtimePreview0; RealtimePreview1; RTBOfflinePreview; RTBSnapshotYUV0; RTBSnapshotYUV1; RTBOfflineSnapshot; RTBJpegEncode | None |
| UsecaseVRVideo | RealtimeVideo0; RealtimeVideo1; VRfflineVideo | None |
| UsecaseVR | RealtimeVRPreview0; RealtimeVRPreview1; VROfflinePreview; VRSnapshotYUV0; VRSnapshotYUV1; VROfflineSnapshot | None |
| UsecaseSAT | RealtimePreviewSAT0; RealtimePreviewSAT1; SATOfflinePreview; SATSnapshotYUV0; SATSnapshotYUV1; SATOfflineSnapshot; SATJpegEncode; SATOfflineRAW16 | None |
| UsecaseYUV2JpegReprocessFourStreamsPreviewWithStats | RealtimeYUVAndPreview; ReprocessJPEGFourStreams; ReprocessFourStreamsPreviewAndYUVOut | None |
| UsecaseYUV2JpegReprocessFiveStreamsPreviewAndCallbackWithStats | RealtimeYUVAndPreviewAndCallback; ReprocessJPEGFiveStreams1; ReprocessJPEGFiveStreams2 | None |
| UsecaseYUVRawReprocessFourStreams | RealtimeYUVRawReprocess4Streams; ReprocessYuvRaw4Streams | None |
| UsecaseYUV2YUVReprocessFiveStreamsPreviewCallbackAndRaw | RealtimeYUVPreviewCallbackAndRaw; ReprocessYUVFiveStreams1; ReprocessYUVFiveStreams2 | None |
| UsecaseYUV2YUVReprocessFourStreamsWithPreviewMoreDownscale | RealtimeYUV2YUVReprocess4StreamsMoreDownscale; ReprocessYuvToYuv4StreamsMoreDownscale | None |
| UsecaseYUV2YUVReprocessFourStreamsWithPreview | RealtimeYUVReprocess4Streams; ReprocessYuv4Streams1; ReprocessYuv4Streams2 | None |
| UsecaseYUV2JpegReprocessFiveStreamsYUVCallbackAndRaw | RealtimePreviewCallbackRawAndJPEG; ReprocessJPEGYUVFiveStreams1; ReprocessJPEGYUVFiveStreams2 | None |

Raw port links, instance IDs, properties and source offsets remain on the build host in `camera-research/evidence/chi-compiled-zsl-graphs.json` and `titan-pipeline-graphs.json`. Reproduce them with the source-controlled decoders listed in the analysis. No binary code is embedded in this document.
