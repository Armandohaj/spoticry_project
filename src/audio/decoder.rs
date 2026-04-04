use std::fs::File;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::errors::Error;

// Información del audio decodificado
pub struct AudioInfo {
    pub sample_rate: u32,
    pub channels: u32,
    pub bits: u32,
}

// Resultado de decodificación: info + todos los bytes PCM
pub struct DecodedAudio {
    pub info: AudioInfo,
    pub samples: Vec<u8>,
}

// Función principal: toma la ruta del MP3 y devuelve los bytes PCM
pub fn decode_mp3(file_path: &str) -> Result<DecodedAudio, String> {
    // Abrir el archivo
    let file = File::open(file_path)
        .map_err(|e| format!("No se pudo abrir el archivo: {}", e))?;

    // Crear el stream de lectura que symphonia entiende
    let media_source = MediaSourceStream::new(Box::new(file), Default::default());

    // Darle una pista a symphonia de que es un MP3
    let mut hint = Hint::new();
    hint.with_extension("mp3");

    // Detectar el formato del archivo automáticamente
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            media_source,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Formato no reconocido: {}", e))?;

    let mut format = probed.format;

    // Obtener la pista de audio principal
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No se encontró pista de audio")?;

    let track_id = track.id;

    // Leer parámetros del codec
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels    = track.codec_params.channels.map(|c| c.count() as u32).unwrap_or(2);

    // Crear el decodificador para esa pista
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("No se pudo crear decodificador: {}", e))?;

    let mut all_samples: Vec<u8> = Vec::new();

    // Decodificar paquete por paquete
    loop {
        // Leer el siguiente paquete del archivo
        let packet = match format.next_packet() {
            Ok(p)                          => p,
            Err(Error::IoError(_))         => break, // fin del archivo
            Err(Error::ResetRequired)      => break,
            Err(e)                         => return Err(format!("Error leyendo paquete: {}", e)),
        };

        // Solo procesar paquetes de nuestra pista
        if packet.track_id() != track_id {
            continue;
        }

        // Decodificar el paquete a samples
        let decoded = match decoder.decode(&packet) {
            Ok(d)  => d,
            Err(_) => continue, // ignorar paquetes con error
        };

        // Convertir los samples a bytes i16 (PCM 16-bit)
        // i16 = -32768 a 32767, es el formato estándar para audio
        let spec   = *decoded.spec();
        let frames = decoded.capacity() as u64;

        let mut sample_buf = SampleBuffer::<i16>::new(frames, spec);
        sample_buf.copy_interleaved_ref(decoded);

        // Convertir cada sample i16 a 2 bytes little-endian
        // Little-endian = byte menos significativo primero
        for sample in sample_buf.samples() {
            let bytes = sample.to_le_bytes();
            all_samples.push(bytes[0]);
            all_samples.push(bytes[1]);
        }
    }

    Ok(DecodedAudio {
        info: AudioInfo {
            sample_rate,
            channels,
            bits: 16, // siempre 16-bit porque usamos i16
        },
        samples: all_samples,
    })
}