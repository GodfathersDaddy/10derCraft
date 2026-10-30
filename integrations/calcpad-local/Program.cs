using System.Globalization;
using System.Text.Json;
using System.Text;
using Calcpad.Core;

CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
CultureInfo.CurrentUICulture = CultureInfo.InvariantCulture;
// The Rust process protocol is UTF-8, independent of Windows console code pages.
Console.InputEncoding = new UTF8Encoding(false);
Console.OutputEncoding = new UTF8Encoding(false);
try
{
    var input = Console.In.ReadToEnd();
    if (input.Length > 262144) throw new ArgumentException("Worksheet request exceeds 256 KiB.");
    using var request = JsonDocument.Parse(input);
    var source = request.RootElement.GetProperty("source").GetString() ?? "";
    // Equations, quoted text and conditional branches are reproducible inputs.
    // External-file directives and macros require separate integration support.
    foreach (var line in source.Split('\n'))
    {
        var text = line.Trim();
        var conditional = text.StartsWith("#if ", StringComparison.OrdinalIgnoreCase) ||
            text.Equals("#else", StringComparison.OrdinalIgnoreCase) ||
            text.Equals("#end if", StringComparison.OrdinalIgnoreCase);
        if (line.Contains('$') || line.Contains('#') && !conditional)
            throw new ArgumentException("Only conditional #if/#else/#end if directives are supported in embedded worksheets.");
    }
    var units = request.RootElement.TryGetProperty("units", out var unitMap)
        ? JsonSerializer.Deserialize<Dictionary<string, string>>(unitMap.GetRawText()) ?? new()
        : new Dictionary<string, string>();
    if (units.Count > 128 || units.Any(p => p.Key.Length > 160 || p.Value.Length > 160))
        throw new ArgumentException("Display unit request exceeds limits.");
    var parser = new ExpressionParser { Debug = true, SolidCraftDisplayUnits = units };
    parser.Parse(source + "\n", true, false);
    if (parser.Errors is { Count: > 0 })
        throw new ArgumentException(string.Join("; ", parser.Errors.Select(error => $"Line {error.SourceLine}: {error.Message}")));
    Console.Write(JsonSerializer.Serialize(new {
        ok = true, backend = "CalcpadCE", results = parser.SolidCraftScalars(), html = parser.HtmlResult,
        display_results = parser.SolidCraftDisplayResults, display_errors = parser.SolidCraftDisplayErrors
    }));
}
catch (Exception error)
{
    Console.Error.WriteLine(error);
    Console.Write(JsonSerializer.Serialize(new { ok = false, error = error.Message }));
    Environment.ExitCode = 1;
}
