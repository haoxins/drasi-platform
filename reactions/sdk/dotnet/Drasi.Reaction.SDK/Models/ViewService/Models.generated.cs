// <auto-generated />
//
// To parse this JSON data, add NuGet 'System.Text.Json' then do one of these:
//
//    using Drasi.Reaction.SDK.Models.ViewService;
//
//    var data = Data.FromJson(jsonString);
//    var header = Header.FromJson(jsonString);
//    var headerItem = HeaderItem.FromJson(jsonString);
//    var viewItem = ViewItem.FromJson(jsonString);
#nullable enable
#pragma warning disable CS8618
#pragma warning disable CS8601
#pragma warning disable CS8603

namespace Drasi.Reaction.SDK.Models.ViewService
{
    using System;
    using System.Collections.Generic;

    using System.Text.Json;
    using System.Text.Json.Serialization;
    using System.Globalization;

    public partial class Data
    {
        [JsonPropertyName("data")]
        public Dictionary<string, object> DataData { get; set; }
    }

    public partial class Header
    {
        [JsonPropertyName("header")]
        public HeaderClass HeaderHeader { get; set; }
    }

    public partial class HeaderClass
    {
        /// <summary>
        /// The sequence number of the event
        /// </summary>
        [JsonPropertyName("sequence")]
        public long Sequence { get; set; }

        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        [JsonPropertyName("state")]
        public string State { get; set; }

        /// <summary>
        /// The time at which the source change was recorded
        /// </summary>
        [JsonPropertyName("timestamp")]
        public long Timestamp { get; set; }
    }

    public partial class HeaderItem
    {
        /// <summary>
        /// The sequence number of the event
        /// </summary>
        [JsonPropertyName("sequence")]
        public long Sequence { get; set; }

        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        [JsonPropertyName("state")]
        public string State { get; set; }

        /// <summary>
        /// The time at which the source change was recorded
        /// </summary>
        [JsonPropertyName("timestamp")]
        public long Timestamp { get; set; }
    }

    public partial class ViewItem
    {
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        [JsonPropertyName("header")]
        public HeaderClass Header { get; set; }

        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        [JsonPropertyName("data")]
        public Dictionary<string, object> Data { get; set; }
    }

    public partial class Data
    {
        public static Data FromJson(string json) => JsonSerializer.Deserialize<Data>(json, Drasi.Reaction.SDK.Models.ViewService.Converter.Settings);
    }

    public partial class Header
    {
        public static Header FromJson(string json) => JsonSerializer.Deserialize<Header>(json, Drasi.Reaction.SDK.Models.ViewService.Converter.Settings);
    }

    public partial class HeaderItem
    {
        public static HeaderItem FromJson(string json) => JsonSerializer.Deserialize<HeaderItem>(json, Drasi.Reaction.SDK.Models.ViewService.Converter.Settings);
    }

    public partial class ViewItem
    {
        public static ViewItem FromJson(string json) => JsonSerializer.Deserialize<ViewItem>(json, Drasi.Reaction.SDK.Models.ViewService.Converter.Settings);
    }

    public static class Serialize
    {
        public static string ToJson(this Data self) => JsonSerializer.Serialize(self, Drasi.Reaction.SDK.Models.ViewService.Converter.Settings);
        public static string ToJson(this Header self) => JsonSerializer.Serialize(self, Drasi.Reaction.SDK.Models.ViewService.Converter.Settings);
        public static string ToJson(this HeaderItem self) => JsonSerializer.Serialize(self, Drasi.Reaction.SDK.Models.ViewService.Converter.Settings);
        public static string ToJson(this ViewItem self) => JsonSerializer.Serialize(self, Drasi.Reaction.SDK.Models.ViewService.Converter.Settings);
    }

    internal static class Converter
    {
        public static readonly JsonSerializerOptions Settings = new(JsonSerializerDefaults.General)
        {
            Converters =
            {
                new DateOnlyConverter(),
                new TimeOnlyConverter(),
                IsoDateTimeOffsetConverter.Singleton
            },
        };
    }
    
    public class DateOnlyConverter : JsonConverter<DateOnly>
    {
        private readonly string serializationFormat;
        public DateOnlyConverter() : this(null) { }

        public DateOnlyConverter(string? serializationFormat)
        {
                this.serializationFormat = serializationFormat ?? "yyyy-MM-dd";
        }

        public override DateOnly Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
                var value = reader.GetString();
                return DateOnly.Parse(value!);
        }

        public override void Write(Utf8JsonWriter writer, DateOnly value, JsonSerializerOptions options)
                => writer.WriteStringValue(value.ToString(serializationFormat));
    }

    public class TimeOnlyConverter : JsonConverter<TimeOnly>
    {
        private readonly string serializationFormat;

        public TimeOnlyConverter() : this(null) { }

        public TimeOnlyConverter(string? serializationFormat)
        {
                this.serializationFormat = serializationFormat ?? "HH:mm:ss.fff";
        }

        public override TimeOnly Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
                var value = reader.GetString();
                return TimeOnly.Parse(value!);
        }

        public override void Write(Utf8JsonWriter writer, TimeOnly value, JsonSerializerOptions options)
                => writer.WriteStringValue(value.ToString(serializationFormat));
    }

    internal class IsoDateTimeOffsetConverter : JsonConverter<DateTimeOffset>
    {
        public override bool CanConvert(Type t) => t == typeof(DateTimeOffset);

        private const string DefaultDateTimeFormat = "yyyy'-'MM'-'dd'T'HH':'mm':'ss.FFFFFFFK";

        private DateTimeStyles _dateTimeStyles = DateTimeStyles.RoundtripKind;
        private string? _dateTimeFormat;
        private CultureInfo? _culture;

        public DateTimeStyles DateTimeStyles
        {
                get => _dateTimeStyles;
                set => _dateTimeStyles = value;
        }

        public string? DateTimeFormat
        {
                get => _dateTimeFormat ?? string.Empty;
                set => _dateTimeFormat = (string.IsNullOrEmpty(value)) ? null : value;
        }

        public CultureInfo Culture
        {
                get => _culture ?? CultureInfo.CurrentCulture;
                set => _culture = value;
        }

        public override void Write(Utf8JsonWriter writer, DateTimeOffset value, JsonSerializerOptions options)
        {
                string text;


                if ((_dateTimeStyles & DateTimeStyles.AdjustToUniversal) == DateTimeStyles.AdjustToUniversal
                        || (_dateTimeStyles & DateTimeStyles.AssumeUniversal) == DateTimeStyles.AssumeUniversal)
                {
                        value = value.ToUniversalTime();
                }

                text = value.ToString(_dateTimeFormat ?? DefaultDateTimeFormat, Culture);

                writer.WriteStringValue(text);
        }

        public override DateTimeOffset Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
                string? dateText = reader.GetString();

                if (string.IsNullOrEmpty(dateText) == false)
                {
                        if (!string.IsNullOrEmpty(_dateTimeFormat))
                        {
                                return DateTimeOffset.ParseExact(dateText, _dateTimeFormat, Culture, _dateTimeStyles);
                        }
                        else
                        {
                                return DateTimeOffset.Parse(dateText, Culture, _dateTimeStyles);
                        }
                }
                else
                {
                        return default(DateTimeOffset);
                }
        }


        public static readonly IsoDateTimeOffsetConverter Singleton = new IsoDateTimeOffsetConverter();
    }
}
#pragma warning restore CS8618
#pragma warning restore CS8601
#pragma warning restore CS8603
