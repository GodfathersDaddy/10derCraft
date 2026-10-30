// SolidCraft integration extension, compiled inside the MIT-licensed Calcpad Core.
// Export precise real scalars, never rounded display strings or matrix first elements.
using System;
using System.Collections.Generic;
namespace Calcpad.Core;
public partial class MathParser
{
    public Dictionary<string, object> SolidCraftDisplayScalars(Dictionary<string, string> requested, Dictionary<string, string> errors)
    {
        var display = new Dictionary<string, object>(StringComparer.Ordinal);
        foreach (var (name, input) in requested)
        {
            if (!_variables.TryGetValue(name, out var variable) || !variable.IsInitialized ||
                variable.Value is not IScalarValue scalar || !scalar.IsReal)
                continue;
            try
            {
                var text = input.Trim().Replace("·", "*").Replace("Nmm", "N*mm").Replace("Nm", "N*m");
                if (text.Length == 0) continue;
                var target = UnitsParser.Parse(text.AsSpan(), _units, IsUs);
                var targetText = text;
                if (!Unit.IsConsistent(scalar.Units, target) && Unit.IsConsistent(target, Unit.Get("m", IsUs)))
                {
                    // A length choice may specify the length component of a compound
                    // mechanical unit, e.g. Nm -> Nmm. Every candidate is dimension checked.
                    var candidates = new[] { $"({text})^2", $"({text})^3", $"({text})^4", $"N*({text})", $"N/({text})^2", $"N/({text})", $"({text})/s", $"({text})/s^2", $"1/({text})" };
                    foreach (var candidate in candidates)
                    {
                        var unit = UnitsParser.Parse(candidate.AsSpan(), _units, IsUs);
                        if (Unit.IsConsistent(scalar.Units, unit)) { target = unit; targetText = candidate; break; }
                    }
                }
                if (!Unit.IsConsistent(scalar.Units, target))
                {
                    // Preserve the requested factor without changing dimensions:
                    // 2 s displayed with m becomes 2 s * (1/m) * m.
                    var original = scalar.Units?.Text.Replace("·", "*") ?? "";
                    var prefix = original.Length == 0 ? "" : $"({original})*";
                    display[name] = new { value = scalar.Re, unit = $"{prefix}(1/({text}))*({text})", forced_factor = true };
                    continue;
                }
                var value = scalar.Re * (scalar.Units?.ConvertTo(target) ?? 1d);
                if (target.IsTemp && scalar.Units is not null)
                    value += Unit.GetTempUnitsDelta(scalar.Units.Text, target.Text);
                if (!double.IsFinite(value)) throw new ArgumentException("Non-finite display value.");
                display[name] = new { value, unit = targetText };
            }
            catch (Exception error) { errors[name] = error.Message; }
        }
        return display;
    }
    public Dictionary<string, object> SolidCraftScalars()
    {
        var results = new Dictionary<string, object>(StringComparer.Ordinal);
        foreach (var entry in _variables)
            if (entry.Value.IsInitialized && entry.Value.Value is IScalarValue scalar &&
                scalar.IsReal && double.IsFinite(scalar.Re))
                results[entry.Key] = new { value = scalar.Re, unit = scalar.Units?.Text ?? "" };
        return results;
    }
}
public partial class ExpressionParser
{
    private Dictionary<string, object> _solidCraftScalars = new();
    public Dictionary<string, string> SolidCraftDisplayUnits { get; set; } = new();
    public Dictionary<string, object> SolidCraftDisplayResults { get; private set; } = new();
    public Dictionary<string, string> SolidCraftDisplayErrors { get; private set; } = new();
    private void CaptureSolidCraftScalars()
    {
        _solidCraftScalars = _parser.SolidCraftScalars();
        SolidCraftDisplayErrors = new();
        SolidCraftDisplayResults = _parser.SolidCraftDisplayScalars(SolidCraftDisplayUnits, SolidCraftDisplayErrors);
    }
    public Dictionary<string, object> SolidCraftScalars() => _solidCraftScalars;
}
